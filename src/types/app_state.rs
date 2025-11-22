use crate::types::AppConfig;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// アプリケーションの共有状態（AppStateパターン）
#[derive(Debug, Clone)]
pub struct AppState {
    pub db_connection: Arc<DatabaseConnection>,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db_connection: DatabaseConnection, config: AppConfig) -> Self {
        Self {
            db_connection: Arc::new(db_connection),
            config,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &*self.db_connection
    }
}
