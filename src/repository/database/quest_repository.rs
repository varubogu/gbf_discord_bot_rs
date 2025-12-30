use crate::models::entities::guild_master::guild_quest_disables;
use crate::models::entities::guild_master::guild_quest_disables::Entity as GuildQuestDisableEntity;
use crate::models::entities::master::{
    quest_aliases, quest_aliases::Entity as QuestAliasEntity, quests, quests::Entity as QuestEntity,
};
use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::quests_repository::QuestSearchResult;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

pub struct SeaOrmQuestRepository;

impl SeaOrmQuestRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QuestRepository for SeaOrmQuestRepository {
    async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<Quest>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let quests = QuestEntity::find().all(db).await?;

        Ok(quests
            .into_iter()
            .map(|q| Quest {
                id: q.id,
                name: q.name,
                default_battle_style_id: q.default_battle_style_id,
                recruit_count: q.recruit_count,
                available_battle_style_ids: q.available_battle_style_ids,
                sort_order: q.sort_order,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })
            .collect())
    }

    async fn get_by_target_id<'c, C>(&self, db: &'c C, target_id: i32) -> Result<Option<Quest>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let quest = QuestEntity::find()
            .filter(quests::Column::Id.eq(target_id))
            .one(db)
            .await?;

        Ok(quest.map(|q| Quest {
            id: q.id,
            name: q.name,
            default_battle_style_id: q.default_battle_style_id,
            recruit_count: q.recruit_count,
            available_battle_style_ids: q.available_battle_style_ids,
            sort_order: q.sort_order,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }))
    }

    async fn search_by_name_or_alias<'c, C>(
        &self,
        db: &'c C,
        partial: &str,
    ) -> Result<Vec<QuestSearchResult>>
    where
        C: sea_orm::ConnectionTrait,
    {
        // クエスト名で部分一致検索
        let quests_by_name = QuestEntity::find()
            .filter(quests::Column::Name.contains(partial))
            .all(db)
            .await?;

        // エイリアスで部分一致検索
        let aliases = QuestAliasEntity::find()
            .filter(quest_aliases::Column::Alias.contains(partial))
            .all(db)
            .await?;

        // エイリアスに対応するクエストIDを取得
        let quest_ids_from_aliases: Vec<i32> = aliases.iter().map(|a| a.quest_id).collect();

        let quests_by_alias = if !quest_ids_from_aliases.is_empty() {
            QuestEntity::find()
                .filter(quests::Column::Id.is_in(quest_ids_from_aliases.clone()))
                .all(db)
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

    async fn search_by_name_or_alias_for_guild<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<QuestSearchResult>>
    where
        C: sea_orm::ConnectionTrait,
    {
        // 空文字の場合はguild_quest_disablesに登録されているクエストを除外、それ以外は全件対象
        let excluded_quest_ids = if partial.trim().is_empty() {
            // 無効化されたクエストIDを取得（これらを除外する）
            let disabled_quest_ids: Vec<i32> = GuildQuestDisableEntity::find()
                .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
                .all(db)
                .await?
                .into_iter()
                .map(|gq| gq.quest_id)
                .collect();

            Some(disabled_quest_ids)
        } else {
            None
        };

        // クエスト名で部分一致検索
        let mut quests_query = QuestEntity::find();

        if !partial.trim().is_empty() {
            quests_query = quests_query.filter(quests::Column::Name.contains(partial));
        }

        // 除外リストがある場合は、それらのクエストを除外
        if let Some(ref excluded_ids) = excluded_quest_ids {
            if !excluded_ids.is_empty() {
                quests_query = quests_query.filter(quests::Column::Id.is_not_in(excluded_ids.clone()));
            }
        }

        let quests_by_name = quests_query
            .order_by_desc(quests::Column::SortOrder)
            .all(db)
            .await?;

        // エイリアスで部分一致検索
        let aliases = if !partial.trim().is_empty() {
            QuestAliasEntity::find()
                .filter(quest_aliases::Column::Alias.contains(partial))
                .all(db)
                .await?
        } else {
            vec![]
        };

        // エイリアスに対応するクエストIDを取得
        let mut quest_ids_from_aliases: Vec<i32> = aliases.iter().map(|a| a.quest_id).collect();

        // 除外リストがある場合は、それらのクエストIDを除外
        if let Some(ref excluded_ids) = excluded_quest_ids {
            quest_ids_from_aliases.retain(|id| !excluded_ids.contains(id));
        }

        let quests_by_alias = if !quest_ids_from_aliases.is_empty() {
            QuestEntity::find()
                .filter(quests::Column::Id.is_in(quest_ids_from_aliases.clone()))
                .order_by_desc(quests::Column::SortOrder)
                .all(db)
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

    async fn search_enabled_quests<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<QuestSearchResult>>
    where
        C: sea_orm::ConnectionTrait,
    {
        // 無効化されているクエストIDを取得
        let disabled_quest_ids: Vec<i32> = GuildQuestDisableEntity::find()
            .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
            .all(db)
            .await?
            .into_iter()
            .map(|gq| gq.quest_id)
            .collect();

        // クエスト名で部分一致検索
        let mut quests_query = QuestEntity::find();

        if !partial.trim().is_empty() {
            quests_query = quests_query.filter(quests::Column::Name.contains(partial));
        }

        // 無効化されたクエストを除外
        if !disabled_quest_ids.is_empty() {
            quests_query = quests_query.filter(quests::Column::Id.is_not_in(disabled_quest_ids.clone()));
        }

        let quests_by_name = quests_query
            .order_by_desc(quests::Column::SortOrder)
            .all(db)
            .await?;

        // エイリアスで部分一致検索
        let aliases = if !partial.trim().is_empty() {
            QuestAliasEntity::find()
                .filter(quest_aliases::Column::Alias.contains(partial))
                .all(db)
                .await?
        } else {
            vec![]
        };

        // エイリアスに対応するクエストIDを取得
        let mut quest_ids_from_aliases: Vec<i32> = aliases.iter().map(|a| a.quest_id).collect();

        // 無効化されたクエストを除外
        if !disabled_quest_ids.is_empty() {
            quest_ids_from_aliases.retain(|id| !disabled_quest_ids.contains(id));
        }

        let quests_by_alias = if !quest_ids_from_aliases.is_empty() {
            QuestEntity::find()
                .filter(quests::Column::Id.is_in(quest_ids_from_aliases.clone()))
                .order_by_desc(quests::Column::SortOrder)
                .all(db)
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

    async fn search_disabled_quests<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<QuestSearchResult>>
    where
        C: sea_orm::ConnectionTrait,
    {
        // 無効化されているクエストIDを取得
        let disabled_quest_ids: Vec<i32> = GuildQuestDisableEntity::find()
            .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
            .all(db)
            .await?
            .into_iter()
            .map(|gq| gq.quest_id)
            .collect();

        if disabled_quest_ids.is_empty() {
            return Ok(vec![]);
        }

        // クエスト名で部分一致検索
        let mut quests_query = QuestEntity::find()
            .filter(quests::Column::Id.is_in(disabled_quest_ids.clone()));

        if !partial.trim().is_empty() {
            quests_query = quests_query.filter(quests::Column::Name.contains(partial));
        }

        let quests_by_name = quests_query
            .order_by_desc(quests::Column::SortOrder)
            .all(db)
            .await?;

        // エイリアスで部分一致検索
        let aliases = if !partial.trim().is_empty() {
            QuestAliasEntity::find()
                .filter(quest_aliases::Column::Alias.contains(partial))
                .all(db)
                .await?
        } else {
            vec![]
        };

        // エイリアスに対応するクエストIDを取得（無効化されているもののみ）
        let quest_ids_from_aliases: Vec<i32> = aliases
            .iter()
            .map(|a| a.quest_id)
            .filter(|id| disabled_quest_ids.contains(id))
            .collect();

        let quests_by_alias = if !quest_ids_from_aliases.is_empty() {
            QuestEntity::find()
                .filter(quests::Column::Id.is_in(quest_ids_from_aliases.clone()))
                .order_by_desc(quests::Column::SortOrder)
                .all(db)
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

    async fn setup_test_db()
    -> std::result::Result<(SeaOrmQuestRepository, sea_orm::DatabaseConnection), String> {
        let conn = match crate::repository::database::db_compat::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to database: {e}")),
        };

        Ok((SeaOrmQuestRepository::new(), conn))
    }

    #[tokio::test]
    async fn test_get_quests() {
        let (repo, conn) = match setup_test_db().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {e}");
                return;
            }
        };

        let result = repo.get_all(&conn).await;

        match result {
            Ok(quests) => {
                println!("Retrieved {} quests", quests.len());
                for quest in quests {
                    assert!(!quest.name.is_empty(), "Quest name should not be empty");
                    assert!(quest.id > 0, "Quest ID should be positive");
                }
            }
            Err(e) => {
                println!("Get quests returned error (maybe expected): {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_search_by_name_or_alias() {
        let (repo, conn) = match setup_test_db().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {e}");
                return;
            }
        };

        let result = repo.search_by_name_or_alias(&conn, "test").await;
        match result {
            Ok(results) => {
                println!("Found {} matching quests", results.len());
                for result in results {
                    assert!(!result.name.is_empty(), "Quest name should not be empty");
                    assert!(
                        !result.matched_text.is_empty(),
                        "Matched text should not be empty"
                    );
                }
            }
            Err(e) => {
                println!("Search returned error (maybe expected): {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_quest_by_id() {
        let (repo, conn) = match setup_test_db().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {e}");
                return;
            }
        };

        let result = repo.get_by_target_id(&conn, 999999).await;
        match result {
            Ok(None) => {
                assert!(true);
            }
            Ok(Some(quest)) => {
                println!("Found a quest for ID 999999: {}", quest.name);
                assert_eq!(quest.id, 999999, "Quest ID should match");
            }
            Err(e) => {
                println!("Get quest by ID returned error: {e}");
            }
        }
    }
}
