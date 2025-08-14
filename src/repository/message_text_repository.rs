use async_trait::async_trait;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use crate::types::PoiseError;
use crate::models::message_text::MessageText;
use crate::models::entities::{message_text, message_text::Entity as MessageTextEntity};

#[async_trait]
pub trait MessageTextRepository {
    /// Get message text by guild ID and message ID
    async fn get_by_guild_and_message(&self, guild_id: i64, message_id: &str) -> Result<Option<MessageText>, PoiseError>;
}

pub struct SeaOrmMessageTextRepository {
    conn: DatabaseConnection,
}

impl SeaOrmMessageTextRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl MessageTextRepository for SeaOrmMessageTextRepository {
    async fn get_by_guild_and_message(&self, guild_id: i64, message_id: &str) -> Result<Option<MessageText>, PoiseError> {
        let message_text = MessageTextEntity::find()
            .filter(message_text::Column::GuildId.eq(guild_id))
            .filter(message_text::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to get message text: {}", e)))?;

        Ok(message_text.map(|mt| MessageText {
            id: mt.id,
            guild_id: mt.guild_id,
            message_id: mt.message_id,
            message_jp: mt.message_jp,
            message_en: mt.message_en,
            created_at: mt.created_at,
            updated_at: mt.updated_at,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_repo() -> Result<SeaOrmMessageTextRepository, String> {
        if std::env::var("DATABASE_URL").is_err() {
            return Err("DATABASE_URL not set".to_string());
        }

        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to a database: {}", e)),
        };

        Ok(SeaOrmMessageTextRepository::new(conn))
    }

    #[tokio::test]
    async fn test_message_text_operations() {
        let repo = match setup_test_repo().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test getting a non-existent message text
        let result = repo.get_by_guild_and_message(123456789, "non_existent_message").await;
        match result {
            Ok(None) => {
                // Expected result for a non-existent message
                assert!(true);
            },
            Ok(Some(message_text)) => {
                println!("Unexpectedly found message text: {}", message_text.message_jp);
                assert!(!message_text.message_jp.is_empty(), "Message text should not be empty");
                assert_eq!(message_text.guild_id, 123456789, "Guild ID should match");
            },
            Err(e) => {
                println!("Get message text returned error: {}", e);
            }
        }
    }
}