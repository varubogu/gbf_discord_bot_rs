//! ユーザー希望クエストリポジトリのSeaORM実装

use crate::models::entities::guild_master::user_desired_quests;
use crate::repository::auto_recruitment::UserDesiredQuestRepository as UserDesiredQuestRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// ユーザー希望クエストリポジトリのSeaORM実装
pub struct SeaOrmUserDesiredQuestRepository;

#[async_trait]
impl UserDesiredQuestRepositoryTrait for SeaOrmUserDesiredQuestRepository {
    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<user_desired_quests::Model>> {
        debug!(guild_id, user_id, "ユーザーの希望クエストを取得します");

        let result = user_desired_quests::Entity::find()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::UserId.eq(user_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, "希望クエストの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            count = result.len(),
            "希望クエストを取得しました"
        );
        Ok(result)
    }

    async fn find_users_by_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Vec<user_desired_quests::Model>> {
        debug!(
            guild_id,
            quest_id, "クエストを希望しているユーザーを取得します"
        );

        let result = user_desired_quests::Entity::find()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::QuestId.eq(quest_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "希望クエストの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            quest_id,
            count = result.len(),
            "クエストを希望しているユーザーを取得しました"
        );
        Ok(result)
    }

    async fn find_users_by_quests(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_ids: Vec<i32>,
    ) -> Result<Vec<user_desired_quests::Model>> {
        debug!(
            guild_id,
            ?quest_ids,
            "複数クエストを希望しているユーザーを取得します"
        );

        let result = user_desired_quests::Entity::find()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::QuestId.is_in(quest_ids.clone()))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, ?quest_ids, "希望クエストの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "複数クエストを希望しているユーザーを取得しました"
        );
        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
        battle_style_id: i32,
    ) -> Result<user_desired_quests::Model> {
        debug!(
            guild_id,
            user_id, quest_id, battle_style_id, "希望クエストを追加します"
        );

        let now = chrono::Utc::now();
        let active_model = user_desired_quests::ActiveModel {
            guild_id: Set(guild_id),
            user_id: Set(user_id),
            quest_id: Set(quest_id),
            battle_style_id: Set(battle_style_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, user_id, quest_id, battle_style_id, "希望クエストの追加に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            user_id, quest_id, battle_style_id, "希望クエストを追加しました"
        );
        Ok(result)
    }

    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
        battle_style_id: i32,
    ) -> Result<u64> {
        debug!(
            guild_id,
            user_id, quest_id, battle_style_id, "希望クエストを削除します"
        );

        let result = user_desired_quests::Entity::delete_many()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::UserId.eq(user_id))
            .filter(user_desired_quests::Column::QuestId.eq(quest_id))
            .filter(user_desired_quests::Column::BattleStyleId.eq(battle_style_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, quest_id, battle_style_id, "希望クエストの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            quest_id,
            battle_style_id,
            deleted_count = result.rows_affected,
            "希望クエストを削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_styles(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
    ) -> Result<u64> {
        debug!(
            guild_id,
            user_id, quest_id, "希望クエストを全属性削除します"
        );

        let result = user_desired_quests::Entity::delete_many()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::UserId.eq(user_id))
            .filter(user_desired_quests::Column::QuestId.eq(quest_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, quest_id, "希望クエストの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            quest_id,
            deleted_count = result.rows_affected,
            "希望クエストを全属性削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<u64> {
        debug!(
            guild_id,
            user_id, "ユーザーの全ての希望クエストを削除します"
        );

        let result = user_desired_quests::Entity::delete_many()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .filter(user_desired_quests::Column::UserId.eq(user_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, "希望クエストの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            deleted_count = result.rows_affected,
            "希望クエストを削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全ての希望クエストを削除します");

        let result = user_desired_quests::Entity::delete_many()
            .filter(user_desired_quests::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "希望クエストの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "希望クエストを削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmUserDesiredQuestRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmUserDesiredQuestRepository {
    pub fn new() -> Self {
        Self
    }
}
