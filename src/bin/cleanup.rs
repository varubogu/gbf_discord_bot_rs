use std::{env, path::Path};

use gbf_discord_bot_rs::di::Repositories;
use gbf_discord_bot_rs::infrastructure::database::connection::sea_orm_connection::DatabaseConnectionManager;
use gbf_discord_bot_rs::services::maintenance::DataCleanupService;
use sea_orm::TransactionTrait;
use tracing::{error, info};

/// データクリーンアップバッチのエントリーポイント
///
/// 環境変数からDB接続情報を取得し、DataCleanupServiceを実行する。
/// 毎日深夜3時にcronで自動実行される。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .env.maintenanceファイルを読み込み（開発環境用）
    // 本番環境では環境変数が直接設定されるため、エラーは無視
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

    // リポジトリ初期化
    let repositories = Repositories::new();

    // クリーンアップサービス初期化
    let cleanup_service = DataCleanupService::new(
        repositories.battle_recruitments,
        repositories.notification,
        repositories.scheduled_task,
    );

    // トランザクション開始
    let txn = db.begin().await.map_err(|e| {
        error!(error = %e, "トランザクション開始に失敗しました");
        e
    })?;

    // クリーンアップ実行
    match cleanup_service.execute(&txn).await {
        Ok(stats) => {
            // トランザクションコミット
            txn.commit().await.map_err(|e| {
                error!(error = %e, "トランザクションコミットに失敗しました");
                e
            })?;

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
            // エラー時はトランザクションロールバック（自動）
            error!(error = %e, "データクリーンアップに失敗しました");
            Err(e.into())
        }
    }
}
