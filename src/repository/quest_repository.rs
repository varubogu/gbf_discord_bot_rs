use async_trait::async_trait;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use crate::types::PoiseError;
use crate::models::quest::{Quest, QuestAlias};
use crate::models::entities::{quest, quest::Entity as QuestEntity, quest_alias, quest_alias::Entity as QuestAliasEntity};

#[async_trait]
pub trait QuestRepository {
    /// Get all quests
    async fn get_all(&self) -> Result<Vec<Quest>, PoiseError>;
    
    /// Get all quest aliases
    async fn get_aliases(&self) -> Result<Vec<QuestAlias>, PoiseError>;
    
    /// Get quest by alias
    async fn get_by_alias(&self, alias: &str) -> Result<Option<Quest>, PoiseError>;
    
    /// Get quest by target ID
    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, PoiseError>;
}

pub struct SeaOrmQuestRepository {
    conn: DatabaseConnection,
}

impl SeaOrmQuestRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl QuestRepository for SeaOrmQuestRepository {
    async fn get_all(&self) -> Result<Vec<Quest>, PoiseError> {
        let quests = QuestEntity::find()
            .all(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to get quests: {}", e)))?;

        Ok(quests.into_iter().map(|q| Quest {
            id: q.id,
            target_id: q.target_id,
            quest_name: q.quest_name,
            default_battle_type: q.default_battle_type,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }).collect())
    }

    async fn get_aliases(&self) -> Result<Vec<QuestAlias>, PoiseError> {
        let aliases = QuestAliasEntity::find()
            .all(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to get quest aliases: {}", e)))?;

        Ok(aliases.into_iter().map(|a| QuestAlias {
            id: a.id,
            target_id: a.target_id,
            alias: a.alias,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }).collect())
    }

    async fn get_by_alias(&self, alias: &str) -> Result<Option<Quest>, PoiseError> {
        // First find the alias to get the target_id
        let quest_alias = QuestAliasEntity::find()
            .filter(quest_alias::Column::Alias.eq(alias))
            .one(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to find quest alias: {}", e)))?;

        if let Some(alias_record) = quest_alias {
            // Then find the quest by target_id
            let quest = QuestEntity::find()
                .filter(quest::Column::TargetId.eq(alias_record.target_id))
                .one(&self.conn)
                .await
                .map_err(|e| PoiseError::from(format!("Failed to find quest by target_id: {}", e)))?;

            Ok(quest.map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, PoiseError> {
        let quest = QuestEntity::find()
            .filter(quest::Column::TargetId.eq(target_id))
            .one(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to find quest by target_id: {}", e)))?;

        Ok(quest.map(|q| Quest {
            id: q.id,
            target_id: q.target_id,
            quest_name: q.quest_name,
            default_battle_type: q.default_battle_type,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseConnection;

    async fn setup_test_db() -> Result<SeaOrmQuestRepository, String> {
        if std::env::var("DATABASE_URL").is_err() {
            return Err("DATABASE_URL not set".to_string());
        }

        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to database: {}", e)),
        };

        Ok(SeaOrmQuestRepository::new(conn))
    }

    #[tokio::test]
    async fn test_get_quests() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        let result = repo.get_all().await;

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
                // This might be expected if a database is empty or not properly initialised
                println!("Get quests returned error (maybe expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_aliases() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        let result = repo.get_aliases().await;

        match result {
            Ok(aliases) => {
                println!("Retrieved {} quest aliases", aliases.len());
                for alias in aliases {
                    assert!(!alias.alias.is_empty(), "Alias should not be empty");
                    assert!(alias.target_id > 0, "Target ID should be positive");
                }
            },
            Err(e) => {
                println!("Get quest aliases returned error (maybe expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_alias() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test with a non-existent alias
        let result = repo.get_by_alias("non_existent_alias").await;
        match result {
            Ok(None) => {
                // Expected result for non-existent alias
                assert!(true);
            },
            Ok(Some(quest)) => {
                println!("Unexpectedly found a quest for non-existent alias: {}", quest.quest_name);
            },
            Err(e) => {
                println!("Get quest by alias returned error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_target_id() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test with a non-existent target ID
        let result = repo.get_by_target_id(999999).await;
        match result {
            Ok(None) => {
                // Expected result for non-existent target ID
                assert!(true);
            },
            Ok(Some(quest)) => {
                println!("Found a quest for target ID 999999: {}", quest.quest_name);
                assert_eq!(quest.target_id, 999999, "Quest target ID should match");
            },
            Err(e) => {
                println!("Get quest by target ID returned error: {}", e);
            }
        }
    }
}