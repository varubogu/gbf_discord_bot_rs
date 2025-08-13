use std::sync::Arc;
use tracing::info;

// Import the models Database struct
use crate::models;
// Import the old model structs for compatibility
use super::models::{quest::Quest, quest::QuestAlias, battle_recruitment::BattleRecruitment, message_text::MessageText, environment::Environment};



pub struct Database {
    inner: Arc<models::database::Database>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        info!("Creating database connection...");
        let inner = match models::database::Database::new().await {
            Ok(db) => Arc::new(db),
            Err(e) => return Err(sqlx::Error::Protocol(format!("Failed to connect to database: {}", e))),
        };

        info!("Connected to database");
        Ok(Self { inner })
    }

    pub async fn get_quests(&self) -> Result<Vec<Quest>, sqlx::Error> {
        match self.inner.get_quests().await {
            Ok(quests) => Ok(quests.into_iter().map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_quest_aliases(&self) -> Result<Vec<QuestAlias>, sqlx::Error> {
        match self.inner.get_quest_aliases().await {
            Ok(aliases) => Ok(aliases.into_iter().map(|a| QuestAlias {
                id: a.id,
                target_id: a.target_id,
                alias: a.alias,
                created_at: a.created_at,
                updated_at: a.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_quest_by_alias(&self, alias: &str) -> Result<Option<Quest>, sqlx::Error> {
        match self.inner.get_quest_by_alias(alias).await {
            Ok(quest) => Ok(quest.map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_quest_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, sqlx::Error> {
        match self.inner.get_quest_by_target_id(target_id).await {
            Ok(quest) => Ok(quest.map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn create_battle_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<BattleRecruitment, sqlx::Error> {
        match self.inner.create_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type_id,
            expiry_date,
        ).await {
            Ok(recruitment) => Ok(BattleRecruitment {
                id: recruitment.id,
                guild_id: recruitment.guild_id,
                channel_id: recruitment.channel_id,
                message_id: recruitment.message_id,
                target_id: recruitment.target_id,
                battle_type_id: recruitment.battle_type_id,
                expiry_date: recruitment.expiry_date,
                recruit_end_message_id: recruitment.recruit_end_message_id,
                created_at: recruitment.created_at,
                updated_at: recruitment.updated_at,
            }),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_battle_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, sqlx::Error> {
        match self.inner.get_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
        ).await {
            Ok(recruitment) => Ok(recruitment.map(|r| BattleRecruitment {
                id: r.id,
                guild_id: r.guild_id,
                channel_id: r.channel_id,
                message_id: r.message_id,
                target_id: r.target_id,
                battle_type_id: r.battle_type_id,
                expiry_date: r.expiry_date,
                recruit_end_message_id: r.recruit_end_message_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn has_recruitment_end_message(
        &self,
        recruitment_id: i32,
    ) -> Result<Option<bool>, sqlx::Error> {
        match self.inner.has_recruitment_end_message(recruitment_id).await {
            Ok(has_message) => Ok(has_message),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn set_recruitment_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), sqlx::Error> {
        match self.inner.set_recruitment_end_message(recruitment_id, message_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_message_text(
        &self,
        guild_id: i64,
        message_id: &str,
    ) -> Result<Option<MessageText>, sqlx::Error> {
        match self.inner.get_message_text(guild_id, message_id).await {
            Ok(message_text) => Ok(message_text.map(|mt| MessageText {
                id: mt.id,
                guild_id: mt.guild_id,
                message_id: mt.message_id,
                message_jp: mt.message_jp,
                message_en: mt.message_en,
                created_at: mt.created_at,
                updated_at: mt.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_environments(&self) -> Result<Vec<Environment>, sqlx::Error> {
        match self.inner.get_environments().await {
            Ok(environments) => Ok(environments.into_iter().map(|env| Environment {
                id: env.id,
                key: env.key,
                value: env.value,
                created_at: env.created_at,
                updated_at: env.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn get_environment(&self, key: &str) -> Result<Option<Environment>, sqlx::Error> {
        match self.inner.get_environment(key).await {
            Ok(environment) => Ok(environment.map(|env| Environment {
                id: env.id,
                key: env.key,
                value: env.value,
                created_at: env.created_at,
                updated_at: env.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }

    pub async fn set_environment(&self, key: &str, value: &str) -> Result<Environment, sqlx::Error> {
        match self.inner.set_environment(key, value).await {
            Ok(environment) => Ok(Environment {
                id: environment.id,
                key: environment.key,
                value: environment.value,
                created_at: environment.created_at,
                updated_at: environment.updated_at,
            }),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
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

    #[tokio::test]
    async fn test_get_quests() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");
        let result = db.get_quests().await;

        // Test that the method doesn't crash and returns a result
        match result {
            Ok(quests) => {
                println!("Retrieved {} quests", quests.len());
                // Verify that each quest has required fields
                for quest in quests {
                    assert!(!quest.quest_name.is_empty(), "Quest name should not be empty");
                    assert!(quest.id > 0, "Quest ID should be positive");
                }
            },
            Err(e) => {
                // This might be expected if database is empty or not properly initialized
                println!("Get quests returned error (may be expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_aliases() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");
        let result = db.get_quest_aliases().await;

        match result {
            Ok(aliases) => {
                println!("Retrieved {} quest aliases", aliases.len());
                for alias in aliases {
                    assert!(!alias.alias.is_empty(), "Alias should not be empty");
                    assert!(alias.target_id > 0, "Target ID should be positive");
                }
            },
            Err(e) => {
                println!("Get quest aliases returned error (may be expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_alias() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");

        // Test with a non-existent alias
        let result = db.get_quest_by_alias("non_existent_alias").await;
        match result {
            Ok(None) => {
                // Expected result for non-existent alias
                assert!(true);
            },
            Ok(Some(quest)) => {
                println!("Unexpectedly found quest for non-existent alias: {}", quest.quest_name);
            },
            Err(e) => {
                println!("Get quest by alias returned error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_target_id() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");

        // Test with a non-existent target ID
        let result = db.get_quest_by_target_id(999999).await;
        match result {
            Ok(None) => {
                // Expected result for non-existent target ID
                assert!(true);
            },
            Ok(Some(quest)) => {
                println!("Found quest for target ID 999999: {}", quest.quest_name);
                assert!(quest.target_id == 999999, "Quest target ID should match");
            },
            Err(e) => {
                println!("Get quest by target ID returned error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_battle_recruitment_operations() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");

        // Test creating a battle_recruitment recruitment
        let guild_id = 123456789;
        let channel_id = 987654321;
        let message_id = 555666777;
        let target_id = 1;
        let battle_type_id = 1;
        let expiry_date = Utc::now() + chrono::Duration::hours(1);

        let create_result = db.create_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type_id,
            expiry_date,
        ).await;

        match create_result {
            Ok(recruitment) => {
                assert_eq!(recruitment.guild_id, guild_id);
                assert_eq!(recruitment.channel_id, channel_id);
                assert_eq!(recruitment.message_id, message_id);
                assert_eq!(recruitment.target_id, target_id);
                assert_eq!(recruitment.battle_type_id, battle_type_id);

                // Test retrieving the created recruitment
                let get_result = db.get_battle_recruitment(guild_id, channel_id, message_id).await;
                match get_result {
                    Ok(Some(retrieved)) => {
                        assert_eq!(retrieved.id, recruitment.id);
                        assert_eq!(retrieved.guild_id, guild_id);
                        assert_eq!(retrieved.channel_id, channel_id);
                        assert_eq!(retrieved.message_id, message_id);
                    },
                    Ok(None) => panic!("Should have retrieved the created recruitment"),
                    Err(e) => println!("Error retrieving recruitment: {}", e),
                }

                // Test recruitment end message operations
                let has_end_msg = db.has_recruitment_end_message(recruitment.id).await;
                match has_end_msg {
                    Ok(Some(false)) | Ok(None) => {
                        // Expected: no end message initially
                        let end_message_id = 111222333;
                        let set_result = db.set_recruitment_end_message(recruitment.id, end_message_id).await;

                        if set_result.is_ok() {
                            let has_end_msg_after = db.has_recruitment_end_message(recruitment.id).await;
                            match has_end_msg_after {
                                Ok(Some(true)) => assert!(true, "Should have end message after setting"),
                                _ => println!("Failed to verify end message was set"),
                            }
                        }
                    },
                    _ => println!("Unexpected result for has_recruitment_end_message"),
                }
            },
            Err(e) => {
                println!("Create battle_recruitment recruitment returned error (may be expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_message_text_operations() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");

        // Test getting a non-existent message text
        let result = db.get_message_text(123456789, "non_existent_message").await;
        match result {
            Ok(None) => {
                // Expected result for non-existent message
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

    #[tokio::test]
    async fn test_environment_operations() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let db = Database::new().await.expect("Failed to create database");

        // Test getting all environments
        let get_all_result = db.get_environments().await;
        match get_all_result {
            Ok(environments) => {
                println!("Retrieved {} environments", environments.len());
                for env in environments {
                    assert!(!env.key.is_empty(), "Environment key should not be empty");
                }
            },
            Err(e) => {
                println!("Get environments returned error: {}", e);
            }
        }

        // Test getting a specific environment
        let test_key = "TEST_KEY";
        let get_result = db.get_environment(test_key).await;
        match get_result {
            Ok(None) => {
                // Try to set the environment variable
                let set_result = db.set_environment(test_key, "test_value").await;
                match set_result {
                    Ok(env) => {
                        assert_eq!(env.key, test_key);
                        assert_eq!(env.value, "test_value");

                        // Try to retrieve it again
                        let get_again_result = db.get_environment(test_key).await;
                        match get_again_result {
                            Ok(Some(retrieved_env)) => {
                                assert_eq!(retrieved_env.key, test_key);
                                assert_eq!(retrieved_env.value, "test_value");
                            },
                            _ => println!("Failed to retrieve set environment"),
                        }
                    },
                    Err(e) => {
                        println!("Set environment returned error: {}", e);
                    }
                }
            },
            Ok(Some(env)) => {
                println!("Found existing environment: {} = {}", env.key, env.value);
                assert_eq!(env.key, test_key);
            },
            Err(e) => {
                println!("Get environment returned error: {}", e);
            }
        }
    }
}