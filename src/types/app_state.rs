use crate::di::{AppMessageService, Repositories, create_message_service};
use crate::types::AppConfig;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// アプリケーションの共有状態（AppStateパターン）
#[derive(Debug, Clone)]
pub struct AppState {
    /// Guild ロール用DB接続（通常のコマンド実行用、RLS適用）
    pub guild_db: Arc<DatabaseConnection>,
    /// System ロール用DB接続（スケジューラー用、RLS適用なし）
    pub system_db: Arc<DatabaseConnection>,
    /// Global ロール用DB接続（マスターデータ更新用、RLS適用なし）
    pub global_db: Arc<DatabaseConnection>,
    pub config: AppConfig,
    /// メッセージサービス
    pub message_service: Arc<AppMessageService>,
    /// リポジトリコンテナ
    pub repositories: Repositories,
}

impl AppState {
    pub fn new(
        guild_db: DatabaseConnection,
        system_db: DatabaseConnection,
        global_db: DatabaseConnection,
        config: AppConfig,
    ) -> Self {
        let guild_db = Arc::new(guild_db);
        let system_db = Arc::new(system_db);
        let global_db = Arc::new(global_db);

        // リポジトリコンテナを初期化
        let repositories =
            Repositories::new(guild_db.clone(), system_db.clone(), global_db.clone());

        Self {
            guild_db,
            system_db,
            global_db,
            config,
            message_service: Arc::new(create_message_service()),
            repositories,
        }
    }

    /// Guild ロール用DB接続を取得（通常のコマンド実行用）
    pub fn guild_db(&self) -> &DatabaseConnection {
        &self.guild_db
    }

    /// System ロール用DB接続を取得（スケジューラー用）
    pub fn system_db(&self) -> &DatabaseConnection {
        &self.system_db
    }

    /// Global ロール用DB接続を取得（マスターデータ更新用）
    pub fn global_db(&self) -> &DatabaseConnection {
        &self.global_db
    }

    /// メッセージサービスを取得
    pub fn message_service(&self) -> &AppMessageService {
        &self.message_service
    }
}
