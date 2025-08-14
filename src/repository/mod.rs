pub mod battle_recruitment_repository;
pub mod quest_repository;
pub mod message_text_repository;
pub mod environment_repository;

use tracing::info;

// Import repository traits
use battle_recruitment_repository::{BattleRecruitmentRepository, SeaOrmBattleRecruitmentRepository};
use quest_repository::{QuestRepository, SeaOrmQuestRepository};
use message_text_repository::{MessageTextRepository, SeaOrmMessageTextRepository};
use environment_repository::{EnvironmentRepository, SeaOrmEnvironmentRepository};

pub struct Database {
    pub quest: Box<dyn QuestRepository + Send + Sync>,
    pub battle_recruitment: Box<dyn BattleRecruitmentRepository + Send + Sync>,
    pub message_text: Box<dyn MessageTextRepository + Send + Sync>,
    pub environment: Box<dyn EnvironmentRepository + Send + Sync>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        info!("Creating database connection...");
        
        // Get database connection
        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(sqlx::Error::Protocol(format!("Failed to connect to database: {}", e))),
        };

        info!("Connected to database");
        Ok(Self {
            quest: Box::new(SeaOrmQuestRepository::new(conn.clone())),
            battle_recruitment: Box::new(SeaOrmBattleRecruitmentRepository::new(conn.clone())),
            message_text: Box::new(SeaOrmMessageTextRepository::new(conn.clone())),
            environment: Box::new(SeaOrmEnvironmentRepository::new(conn)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_database_new() {
        // Test database creation
        // Note: This test will be skipped if DATABASE_URL is not set
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let result = Database::new().await;
        assert!(result.is_ok(), "Database creation should succeed with valid connection");
    }

}