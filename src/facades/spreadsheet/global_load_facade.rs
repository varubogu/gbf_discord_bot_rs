use crate::facades::scheduler::SchedulerFacade;
use crate::services::spreadsheet::global_loader_service::{
    GlobalLoaderService, GlobalLoaderServiceImpl,
};
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use std::env;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

/// グローバルスプレッドシート読み込み結果
#[derive(Debug)]
pub struct GlobalLoadResult {
    /// データ読み込みが成功したか
    pub data_loaded: bool,
    /// スケジュール生成が成功したか
    pub schedule_generated: bool,
    /// スケジュール生成エラーメッセージ（失敗時のみ）
    pub schedule_error: Option<String>,
}

/// グローバルスプレッドシートからデータを読み込み
///
/// # 引数
/// - `app_state`: アプリケーション状態
///
/// # 戻り値
/// - `Result<GlobalLoadResult>`: 処理結果
#[instrument(level = "debug", skip(app_state))]
pub async fn execute_global_load(app_state: &AppState) -> Result<GlobalLoadResult> {
    info!("GlobalLoadFacade::execute_global_load - グローバルスプレッドシート読み込み処理を開始");

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
            info!("グローバルスプレッドシート読み込み処理完了");

            // スケジュール生成を自動実行
            info!("イベントスケジュールから通知スケジュールを生成します");
            let app_state_arc = Arc::new(app_state.clone());
            let scheduler_facade = SchedulerFacade::new(app_state_arc);

            let (schedule_generated, schedule_error) = match scheduler_facade.generate_schedules().await {
                Ok(_) => {
                    info!("通知スケジュール生成完了");
                    (true, None)
                }
                Err(e) => {
                    warn!(error = %e, "通知スケジュール生成に失敗しました（データ読み込みは成功）");
                    (false, Some(e.to_string()))
                }
            };

            Ok(GlobalLoadResult {
                data_loaded: true,
                schedule_generated,
                schedule_error,
            })
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "グローバルスプレッドシート読み込み処理失敗");
            Err(e)
        }
    }
}
