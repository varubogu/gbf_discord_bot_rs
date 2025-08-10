use std::env;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, DbErr};
use tracing::info;

pub struct Database {
    pub conn: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self, DbErr> {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
            
        info!("Connecting to database...");
        let conn = SeaDatabase::connect(&database_url).await?;
            
        info!("Connected to database");
        Ok(Self { conn })
    }
}