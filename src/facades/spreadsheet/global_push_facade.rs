use crate::services::spreadsheet::global_push_service::{GlobalPushService, GlobalPushServiceImpl};
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// グローバルスプレッドシートにデータを書き込み
///
/// # 引数
/// - `app_state`: アプリケーション状態
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(app_state))]
pub async fn execute_global_push(app_state: &AppState) -> Result<()> {
    info!("GlobalPushFacade::execute_global_push - グローバルスプレッドシート書き込み処理を開始");

    // グローバルデータ書き出しにはGlobalロールを使用
    let txn = app_state.global_db().begin().await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let push_service = GlobalPushServiceImpl::new();

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
            info!("グローバルスプレッドシート書き込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "グローバルスプレッドシート書き込み処理失敗");
            Err(e)
        }
    }
}
