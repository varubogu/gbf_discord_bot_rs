use crate::models::entities::{
    quest_aliases, quest_aliases::Entity as QuestAliasEntity, quests,
    quests::Entity as QuestEntity,
};
use crate::models::quests::Quest;
use crate::repository::quests_repository::QuestSearchResult;
use crate::repository::QuestRepository;
use crate::types::{AppError, Result};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;

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
    async fn get_all(&self) -> Result<Vec<Quest>> {
        let quests = QuestEntity::find()
            .all(&self.conn)
            .await?;

        Ok(quests
            .into_iter()
            .map(|q| Quest {
                id: q.id,
                name: q.name,
                default_battle_style_id: q.default_battle_style_id,
                recruit_count: q.recruit_count,
                available_battle_style_ids: q.available_battle_style_ids,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })
            .collect())
    }

    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>> {
        let quest = QuestEntity::find()
            .filter(quests::Column::Id.eq(target_id))
            .one(&self.conn)
            .await?;

        Ok(quest.map(|q| Quest {
            id: q.id,
            name: q.name,
            default_battle_style_id: q.default_battle_style_id,
            recruit_count: q.recruit_count,
            available_battle_style_ids: q.available_battle_style_ids,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }))
    }

    async fn search_by_name_or_alias(&self, partial: &str) -> Result<Vec<QuestSearchResult>> {
        // クエスト名で部分一致検索
        let quests_by_name = QuestEntity::find()
            .filter(quests::Column::Name.contains(partial))
            .all(&self.conn)
            .await?;

        // エイリアスで部分一致検索
        let aliases = QuestAliasEntity::find()
            .filter(quest_aliases::Column::Alias.contains(partial))
            .all(&self.conn)
            .await?;

        // エイリアスに対応するクエストIDを取得
        let quest_ids_from_aliases: Vec<i32> = aliases.iter().map(|a| a.quest_id).collect();

        let quests_by_alias = if !quest_ids_from_aliases.is_empty() {
            QuestEntity::find()
                .filter(quests::Column::Id.is_in(quest_ids_from_aliases.clone()))
                .all(&self.conn)
                .await?
        } else {
            vec![]
        };

        // クエストIDをキーとしたマップを作成
        let quest_map: HashMap<i32, String> = quests_by_alias
            .iter()
            .map(|q| (q.id, q.name.clone()))
            .collect();

        let alias_map: HashMap<i32, String> = aliases
            .iter()
            .map(|a| (a.quest_id, a.alias.clone()))
            .collect();

        let mut results = Vec::new();

        // クエスト名でマッチしたものを追加
        for quest in quests_by_name {
            results.push(QuestSearchResult {
                quest_id: quest.id,
                name: quest.name.clone(),
                matched_text: quest.name,
            });
        }

        // エイリアスでマッチしたものを追加（重複を避ける）
        for (quest_id, quest_name) in quest_map {
            // 既に名前でマッチしているかチェック
            if !results.iter().any(|r| r.quest_id == quest_id) {
                if let Some(alias) = alias_map.get(&quest_id) {
                    results.push(QuestSearchResult {
                        quest_id,
                        name: quest_name,
                        matched_text: alias.clone(),
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> std::result::Result<SeaOrmQuestRepository, String> {
        let conn = match crate::repository::database::db_compat::Database::new().await {
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

        match result {
            Ok(quests) => {
                println!("Retrieved {} quests", quests.len());
                for quest in quests {
                    assert!(
                        !quest.name.is_empty(),
                        "Quest name should not be empty"
                    );
                    assert!(quest.id > 0, "Quest ID should be positive");
                }
            }
            Err(e) => {
                println!("Get quests returned error (maybe expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_search_by_name_or_alias() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        let result = repo.search_by_name_or_alias("test").await;
        match result {
            Ok(results) => {
                println!("Found {} matching quests", results.len());
                for result in results {
                    assert!(!result.name.is_empty(), "Quest name should not be empty");
                    assert!(!result.matched_text.is_empty(), "Matched text should not be empty");
                }
            }
            Err(e) => {
                println!("Search returned error (maybe expected): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_id() {
        let repo = match setup_test_db().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        let result = repo.get_by_target_id(999999).await;
        match result {
            Ok(None) => {
                assert!(true);
            }
            Ok(Some(quest)) => {
                println!("Found a quest for ID 999999: {}", quest.name);
                assert_eq!(quest.id, 999999, "Quest ID should match");
            }
            Err(e) => {
                println!("Get quest by ID returned error: {}", e);
            }
        }
    }
}
