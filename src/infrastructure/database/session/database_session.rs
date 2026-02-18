use crate::infrastructure::database::connection::build_database_url;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, DbErr};
use tracing::info;

/// 旧Databaseラッパー置換用の互換セッション接続
///
/// 段階移行中のコードでのみ利用し、新規機能では `AppState` 管理接続を優先する。
pub struct DatabaseSession {
    pub conn: DatabaseConnection,
}

impl DatabaseSession {
    pub async fn new() -> Result<Self, DbErr> {
        let database_url = build_database_url().map_err(|e| DbErr::Custom(e.message))?;

        info!("Connecting to a database...");
        let conn = SeaDatabase::connect(&database_url).await?;
        info!("Connected to a database");

        Ok(Self { conn })
    }
}
