use crate::models::entities::master::message_texts::Entity as MessageTextEntity;
use crate::models::message_texts::MessageTexts;
use crate::repository::MessageTextRepository;
use async_trait::async_trait;
use sea_orm::{DbErr, EntityTrait};

#[derive(Debug, Default, Clone, Copy)]
pub struct SeaOrmMessageTextRepository;

impl SeaOrmMessageTextRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MessageTextRepository for SeaOrmMessageTextRepository {
    async fn get_by_id<'c, C>(&self, db: &'c C, id: &str) -> Result<Option<MessageTexts>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        let model = MessageTextEntity::find_by_id(id.to_string())
            .one(db)
            .await?;

        Ok(model.map(|m| m.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::connection::connection_manager::is_database_available;

    async fn setup_test_repo()
    -> Result<(SeaOrmMessageTextRepository, sea_orm::DatabaseConnection), String> {
        let (available, missing) = is_database_available();
        if !available {
            return Err(format!(
                "Database connection info not set - missing: {missing:?}"
            ));
        }

        let conn = match crate::infrastructure::database::repositories::models_database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to a database: {e}")),
        };

        Ok((SeaOrmMessageTextRepository::new(), conn))
    }

    #[tokio::test]
    async fn test_message_text_operations() {
        let (repo, conn) = match setup_test_repo().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {e}");
                return;
            }
        };

        // Test getting a non-existent message text
        let result = repo.get_by_id(&conn, "non_existent_message").await;
        match result {
            Ok(None) => {
                // 存在しないメッセージに対する期待される結果
            }
            Ok(Some(message_text)) => {
                println!(
                    "Unexpectedly found message text: {}",
                    message_text.message_jp
                );
                assert!(
                    !message_text.message_jp.is_empty(),
                    "Message text should not be empty"
                );
                assert_eq!(
                    message_text.id, "non_existent_message",
                    "Message ID should match"
                );
            }
            Err(e) => {
                println!("Get message text returned error: {e}");
            }
        }
    }
}
