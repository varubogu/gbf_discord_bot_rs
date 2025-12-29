use std::{env, path::Path};

use gbf_discord_bot_rs::infrastructure::database::connection::sea_orm_connection::DatabaseConnectionManager;
use gbf_discord_bot_rs::services::maintenance::DataCleanupService;
use tracing::{error, info};
use tracing_subscriber;

/// データクリーンアップバッチのエントリーポイント
///
/// 環境変数からDB接続情報を取得し、DataCleanupServiceを実行する。
/// 毎日深夜3時にcronで自動実行される。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .envファイルを読み込み（開発環境用）
    // 本番環境では環境変数が直接設定されるため、エラーは無視
    // Load environment variables
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env.maintenance");
    dotenv::from_path(dotenv_path).ok();

    // ログ初期化
    tracing_subscriber::fmt::init();

    info!("データクリーンアップバッチを開始します");

    // DB接続
    let manager = DatabaseConnectionManager::new().await.map_err(|e| {
        error!(error = %e, "データベース接続に失敗しました");
        e
    })?;

    let db = manager.connection().clone();

    // クリーンアップサービス初期化
    let cleanup_service = DataCleanupService::new(db);

    // クリーンアップ実行
    match cleanup_service.execute().await {
        Ok(stats) => {
            info!(
                recruitments = stats.deleted_recruitments,
                notifications = stats.deleted_notifications,
                tasks = stats.deleted_tasks,
                cleanup_before = %stats.cleanup_before,
                "データクリーンアップが正常に完了しました"
            );
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "データクリーンアップに失敗しました");
            Err(e.into())
        }
    }
}
