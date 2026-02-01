use crate::events::permission::is_bot_admin_server;
use crate::facades::scheduler::SchedulerFacade;
use crate::services::spreadsheet::global_loader_service::{
    GlobalLoaderService, GlobalLoaderServiceImpl,
};
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
use std::env;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

/// グローバルスプレッドシートからデータを読み込み
///
/// # 引数
/// - `ctx`: PoiseContext
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(ctx))]
pub async fn execute_global_load(ctx: &PoiseContext<'_>) -> Result<()> {
    info!("GlobalLoadFacade::execute_global_load - グローバルスプレッドシート読み込み処理を開始");

    // 管理者専用サーバーかチェック
    let is_admin_server = is_bot_admin_server(ctx).await.map_err(|e| {
        error!("Admin server check failed: {}", e);
        e
    })?;

    if !is_admin_server {
        ctx.say("このコマンドは管理者専用サーバーでのみ実行可能です")
            .await?;
        return Ok(());
    }

    let init_message = "グローバルスプレッドシートからデータ読み込み中...";
    ctx.say(init_message).await?;

    info!(
        "User {} started global spreadsheet load in admin server",
        ctx.author().id
    );

    let app_state = &ctx.data().app_state;
    // グローバルデータ更新にはGlobalロールを使用
    let txn = app_state.global_db().begin().await?;

    let result = async {
        let spreadsheet_id =
            env::var("GLOBAL_SPREADSHEET_ID").map_err(|_| crate::types::AppError::Config {
                message: "環境変数 GLOBAL_SPREADSHEET_ID が設定されていません".to_string(),
            })?;

        let service_account_key_file =
            env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE").map_err(|_| {
                crate::types::AppError::Config {
                    message: "環境変数 GOOGLE_SERVICE_ACCOUNT_KEY_FILE が設定されていません"
                        .to_string(),
                }
            })?;

        // Service層のインスタンス化（静的ディスパッチ）
        let loader_service = GlobalLoaderServiceImpl::new(spreadsheet_id, service_account_key_file);

        // スプレッドシート接続
        loader_service.open_spreadsheet().await?;

        // グローバルデータ読み込み
        let table_data = loader_service.load_global_table_data().await?;

        // データ変換・検証
        let converted_data = loader_service.convert_global_data(table_data).await?;

        // データベース保存
        loader_service
            .save_global_data(&txn, converted_data)
            .await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            ctx.say("グローバルスプレッドシートからデータ読み込み完了")
                .await?;
            info!("グローバルスプレッドシート読み込み処理完了");

            // スケジュール生成を自動実行
            info!("イベントスケジュールから通知スケジュールを生成します");
            let app_state_arc = Arc::new(app_state.clone());
            let scheduler_facade = SchedulerFacade::new(app_state_arc);

            match scheduler_facade.generate_schedules().await {
                Ok(_) => {
                    ctx.say("✅ 通知スケジュールの生成が完了しました").await?;
                    info!("通知スケジュール生成完了");
                }
                Err(e) => {
                    warn!(error = %e, "通知スケジュール生成に失敗しました（データ読み込みは成功）");
                    ctx.say("⚠️ データ読み込みは完了しましたが、スケジュール生成に失敗しました")
                        .await?;
                }
            }

            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "グローバルスプレッドシート読み込み処理失敗");
            ctx.say("グローバルスプレッドシートからデータ読み込み失敗")
                .await?;
            Err(e)
        }
    }
}
