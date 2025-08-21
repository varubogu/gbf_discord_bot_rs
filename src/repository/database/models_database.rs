use crate::services::database::connection::build_database_url;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, DbErr};
use std::env;
use tracing::info;

pub struct Database {
    pub conn: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self, DbErr> {
        // 集約されたdatabase connectionサービスを使用してURLを構築
        let database_url = build_database_url().map_err(|e| DbErr::Custom(e.message))?;

        info!("Connecting to database...");
        let conn = SeaDatabase::connect(&database_url).await?;

        info!("Connected to database");
        Ok(Self { conn })
    }

    // /// DatabaseServiceとの統合メソッド
    // /// utils/database.rsのSeaOrmDatabaseとして利用するための変換メソッド
    // pub fn as_database_service(&self) -> SeaOrmDatabase {
    //     SeaOrmDatabase::new(self.conn.clone())
    // }
}
