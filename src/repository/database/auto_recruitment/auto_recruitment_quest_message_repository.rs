//! 自動募集クエストメッセージリポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitment_quest_messages;
use crate::repository::auto_recruitment::AutoRecruitmentQuestMessageRepository as AutoRecruitmentQuestMessageRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 自動募集クエストメッセージリポジトリのSeaORM実装
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentQuestMessageRepository;

#[async_trait]
impl AutoRecruitmentQuestMessageRepositoryTrait for SeaOrmAutoRecruitmentQuestMessageRepository {
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_quest_messages::Model>> {
        debug!(guild_id, "ギルドの全てのクエストメッセージを取得します");

        let result = auto_recruitment_quest_messages::Entity::find()
            .filter(auto_recruitment_quest_messages::Column::GuildId.eq(guild_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "クエストメッセージの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "クエストメッセージを取得しました"
        );
        Ok(result)
    }

    async fn find_by_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Option<auto_recruitment_quest_messages::Model>> {
        debug!(guild_id, quest_id, "クエストのメッセージを取得します");

        let result = auto_recruitment_quest_messages::Entity::find()
            .filter(auto_recruitment_quest_messages::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_quest_messages::Column::QuestId.eq(quest_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "クエストメッセージの取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn upsert(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        message_id: i64,
    ) -> Result<auto_recruitment_quest_messages::Model> {
        debug!(
            guild_id,
            quest_id, message_id, "クエストメッセージを作成/更新します"
        );

        let now = chrono::Utc::now();
        let existing = self.find_by_quest(txn, guild_id, quest_id).await?;

        let result = if let Some(existing) = existing {
            // 更新
            let mut active_model: auto_recruitment_quest_messages::ActiveModel = existing.into();
            active_model.message_id = Set(message_id);
            active_model.updated_at = Set(now);
            active_model.update(txn).await?
        } else {
            // 新規作成
            let active_model = auto_recruitment_quest_messages::ActiveModel {
                guild_id: Set(guild_id),
                quest_id: Set(quest_id),
                message_id: Set(message_id),
                created_at: Set(now),
                updated_at: Set(now),
            };
            active_model.insert(txn).await?
        };

        debug!(
            guild_id,
            quest_id, message_id, "クエストメッセージを保存しました"
        );
        Ok(result)
    }

    async fn delete(&self, txn: &DatabaseTransaction, guild_id: i64, quest_id: i32) -> Result<u64> {
        debug!(guild_id, quest_id, "クエストメッセージを削除します");

        let result = auto_recruitment_quest_messages::Entity::delete_many()
            .filter(auto_recruitment_quest_messages::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_quest_messages::Column::QuestId.eq(quest_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "クエストメッセージの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            quest_id,
            deleted_count = result.rows_affected,
            "クエストメッセージを削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全てのクエストメッセージを削除します");

        let result = auto_recruitment_quest_messages::Entity::delete_many()
            .filter(auto_recruitment_quest_messages::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "クエストメッセージの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "クエストメッセージを削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmAutoRecruitmentQuestMessageRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmAutoRecruitmentQuestMessageRepository {
    pub fn new() -> Self {
        Self
    }
}
