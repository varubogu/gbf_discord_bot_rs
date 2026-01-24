//! マッチングユーザーリポジトリのSeaORM実装

use crate::models::entities::worker::quest_matching_users;
use crate::repository::auto_recruitment::QuestMatchingUserRepository as QuestMatchingUserRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};
use uuid::Uuid;

/// マッチングユーザーリポジトリのSeaORM実装
pub struct SeaOrmQuestMatchingUserRepository;

#[async_trait]
impl QuestMatchingUserRepositoryTrait for SeaOrmQuestMatchingUserRepository {
    async fn find_active_by_matching(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
    ) -> Result<Vec<quest_matching_users::Model>> {
        debug!(guild_id, %matching_id, "マッチングの参加中ユーザーを取得します");

        let result = quest_matching_users::Entity::find()
            .filter(quest_matching_users::Column::GuildId.eq(guild_id))
            .filter(quest_matching_users::Column::MatchingId.eq(matching_id))
            .filter(quest_matching_users::Column::LeftAt.is_null())
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, %matching_id, "参加中ユーザーの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            %matching_id,
            count = result.len(),
            "参加中ユーザーを取得しました"
        );
        Ok(result)
    }

    async fn find_active_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<quest_matching_users::Model>> {
        debug!(
            guild_id,
            user_id, "ユーザーが参加中のマッチングを取得します"
        );

        let result = quest_matching_users::Entity::find()
            .filter(quest_matching_users::Column::GuildId.eq(guild_id))
            .filter(quest_matching_users::Column::UserId.eq(user_id))
            .filter(quest_matching_users::Column::LeftAt.is_null())
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, "参加中マッチングの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            count = result.len(),
            "参加中マッチングを取得しました"
        );
        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
        battle_style_id: Option<i32>,
    ) -> Result<quest_matching_users::Model> {
        debug!(
            guild_id,
            %matching_id,
            user_id,
            ?battle_style_id,
            "マッチングユーザーを追加します"
        );

        let now = chrono::Utc::now();
        let active_model = quest_matching_users::ActiveModel {
            guild_id: Set(guild_id),
            matching_id: Set(matching_id),
            user_id: Set(user_id),
            battle_style_id: Set(battle_style_id),
            joined_at: Set(now),
            left_at: Set(None),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, %matching_id, user_id, "マッチングユーザーの追加に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            %matching_id,
            user_id,
            "マッチングユーザーを追加しました"
        );
        Ok(result)
    }

    async fn update_battle_style(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
        battle_style_id: Option<i32>,
    ) -> Result<quest_matching_users::Model> {
        debug!(
            guild_id,
            %matching_id,
            user_id,
            ?battle_style_id,
            "属性を更新します"
        );

        let user = quest_matching_users::Entity::find()
            .filter(quest_matching_users::Column::GuildId.eq(guild_id))
            .filter(quest_matching_users::Column::MatchingId.eq(matching_id))
            .filter(quest_matching_users::Column::UserId.eq(user_id))
            .one(txn)
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::RecordNotFound("マッチングユーザーが見つかりません".to_string())
            })?;

        let mut active_model: quest_matching_users::ActiveModel = user.into();
        active_model.battle_style_id = Set(battle_style_id);

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, %matching_id, user_id, "属性更新に失敗しました");
            e
        })?;

        debug!(guild_id, %matching_id, user_id, "属性を更新しました");
        Ok(result)
    }

    async fn leave(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
    ) -> Result<quest_matching_users::Model> {
        debug!(guild_id, %matching_id, user_id, "マッチングから離脱します");

        let user = quest_matching_users::Entity::find()
            .filter(quest_matching_users::Column::GuildId.eq(guild_id))
            .filter(quest_matching_users::Column::MatchingId.eq(matching_id))
            .filter(quest_matching_users::Column::UserId.eq(user_id))
            .one(txn)
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::RecordNotFound("マッチングユーザーが見つかりません".to_string())
            })?;

        let mut active_model: quest_matching_users::ActiveModel = user.into();
        active_model.left_at = Set(Some(chrono::Utc::now()));

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, %matching_id, user_id, "離脱処理に失敗しました");
            e
        })?;

        debug!(guild_id, %matching_id, user_id, "離脱しました");
        Ok(result)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全てのマッチングユーザーを削除します");

        let result = quest_matching_users::Entity::delete_many()
            .filter(quest_matching_users::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチングユーザーの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "マッチングユーザーを削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmQuestMatchingUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmQuestMatchingUserRepository {
    pub fn new() -> Self {
        Self
    }
}
