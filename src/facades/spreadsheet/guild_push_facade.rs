use crate::repository::db_helper::set_current_guild_id;
use crate::services::spreadsheet::guild_push_service::{PushService, PushServiceImpl};
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// サーバー固有スプレッドシートにデータを書き込み
///
/// # 引数
/// - `app_state`: アプリケーション状態
/// - `guild_id`: ギルドID
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(app_state))]
pub async fn execute_push(app_state: &AppState, guild_id: u64) -> Result<()> {
    info!("PushFacade::execute_push - サーバー固有スプレッドシート書き込み処理を開始");

    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let push_service = PushServiceImpl::new();

        // スプレッドシート接続
        PushService::open_spreadsheet(&push_service, guild_id).await?;

        // サーバー固有データ取得
        let data = PushService::load_data(&push_service, guild_id).await?;

        // データ変換・検証
        let converted_data = PushService::convert_data(&push_service, data).await?;

        // スプレッドシート書き込み
        PushService::push_data(&push_service, converted_data, guild_id).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(guild_id = %guild_id, "サーバー固有スプレッドシート書き込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(guild_id = %guild_id, error = %e, "サーバー固有スプレッドシート書き込み処理失敗");
            Err(e)
        }
    }
}
