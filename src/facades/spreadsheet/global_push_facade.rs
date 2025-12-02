use crate::services::permission::is_bot_admin_server;
use crate::services::spreadsheet::global_push_service::{GlobalPushService, GlobalPushServiceImpl};
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// グローバルスプレッドシートにデータを書き込み
///
/// # 引数
/// - `ctx`: PoiseContext
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(ctx))]
pub async fn execute_global_push(ctx: &PoiseContext<'_>) -> Result<()> {
    info!("GlobalPushFacade::execute_global_push - グローバルスプレッドシート書き込み処理を開始");

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

    let init_message = "データベースからグローバルスプレッドシートへ書き込み中...";
    ctx.say(init_message).await?;

    info!(
        "User {} started global spreadsheet push in admin server",
        ctx.author().id
    );

    let app_state = &ctx.data().app_state;
    // グローバルデータ書き出しにはGlobalロールを使用
    let txn = app_state.global_db().begin().await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let mut push_service = GlobalPushServiceImpl::new();

        // スプレッドシート接続
        GlobalPushService::open_spreadsheet(&push_service).await?;

        // グローバルデータ取得
        let data = GlobalPushService::load_global_data(&push_service).await?;

        // データ変換・検証
        let converted_data = GlobalPushService::convert_global_data(&push_service, data).await?;

        // スプレッドシート書き込み
        GlobalPushService::push_global_data(&push_service, converted_data).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            ctx.say("データベースからグローバルスプレッドシートへ書き込み完了")
                .await?;
            info!("グローバルスプレッドシート書き込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "グローバルスプレッドシート書き込み処理失敗");
            ctx.say("データベースからグローバルスプレッドシートへ書き込み失敗")
                .await?;
            Err(e)
        }
    }
}
