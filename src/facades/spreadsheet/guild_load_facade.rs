use crate::repository::db_helper::set_current_guild_id;
use crate::services::spreadsheet::guild_loader_service::{LoaderService, LoaderServiceImpl};
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// サーバー固有スプレッドシートからデータを読み込み
///
/// # 引数
/// - `app_state`: アプリケーション状態
/// - `guild_id`: ギルドID
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(app_state))]
pub async fn execute_load(app_state: &AppState, guild_id: u64) -> Result<()> {
    info!("LoadFacade::execute_load - サーバー固有スプレッドシート読み込み処理を開始");

    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let loader_service = LoaderServiceImpl::new();

        // スプレッドシート接続
        loader_service.open_spreadsheet(guild_id).await?;

        // サーバー固有データ読み込み
        let table_data = loader_service.load_table_data(guild_id).await?;

        // データ変換・検証
        let converted_data = loader_service.convert_data(table_data).await?;

        // データベース保存
        loader_service.save_data(converted_data, guild_id).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(guild_id = %guild_id, "サーバー固有スプレッドシート読み込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(guild_id = %guild_id, error = %e, "サーバー固有スプレッドシート読み込み処理失敗");
            Err(e)
        }
    }
}
