//! マッチングリポジトリのSeaORM実装

use crate::models::entities::worker::quest_matchings;
use crate::repository::auto_recruitment::QuestMatchingRepository as QuestMatchingRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};
use uuid::Uuid;

/// マッチングリポジトリのSeaORM実装
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmQuestMatchingRepository;

#[async_trait]
impl QuestMatchingRepositoryTrait for SeaOrmQuestMatchingRepository {
    async fn find_active_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<quest_matchings::Model>> {
        debug!(guild_id, "ギルドのアクティブなマッチングを取得します");

        let result = quest_matchings::Entity::find()
            .filter(quest_matchings::Column::GuildId.eq(guild_id))
            .filter(quest_matchings::Column::Status.eq("active"))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチングの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "アクティブなマッチングを取得しました"
        );
        Ok(result)
    }

    async fn find_all_active(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<quest_matchings::Model>> {
        debug!("全ギルドのアクティブなマッチングを取得します");

        let result = quest_matchings::Entity::find()
            .filter(quest_matchings::Column::Status.eq("active"))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "マッチングの取得に失敗しました");
                e
            })?;

        debug!(
            count = result.len(),
            "全ギルドのアクティブなマッチングを取得しました"
        );
        Ok(result)
    }

    async fn find_by_schedule(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<quest_matchings::Model>> {
        debug!(
            guild_id,
            quest_id, month, day, hour, "スケジュールでマッチングを検索します"
        );

        let result = quest_matchings::Entity::find()
            .filter(quest_matchings::Column::GuildId.eq(guild_id))
            .filter(quest_matchings::Column::QuestId.eq(quest_id))
            .filter(quest_matchings::Column::ScheduledMonth.eq(month))
            .filter(quest_matchings::Column::ScheduledDay.eq(day))
            .filter(quest_matchings::Column::ScheduledHour.eq(hour))
            .filter(quest_matchings::Column::Status.eq("active"))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "マッチングの検索に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
    ) -> Result<Option<quest_matchings::Model>> {
        debug!(guild_id, %id, "IDでマッチングを取得します");

        let result = quest_matchings::Entity::find()
            .filter(quest_matchings::Column::GuildId.eq(guild_id))
            .filter(quest_matchings::Column::Id.eq(id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, %id, "マッチングの取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<quest_matchings::Model> {
        debug!(
            guild_id,
            quest_id, month, day, hour, "マッチングを作成します"
        );

        let now = chrono::Utc::now();
        let id = Uuid::new_v4();
        let active_model = quest_matchings::ActiveModel {
            guild_id: Set(guild_id),
            id: Set(id),
            quest_id: Set(quest_id),
            scheduled_month: Set(month),
            scheduled_day: Set(day),
            scheduled_hour: Set(hour),
            status: Set("active".to_string()),
            recruitment_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, quest_id, "マッチングの作成に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            quest_id,
            %id,
            "マッチングを作成しました"
        );
        Ok(result)
    }

    async fn update_status(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
        status: &str,
    ) -> Result<quest_matchings::Model> {
        debug!(guild_id, %id, status, "マッチングのステータスを更新します");

        let matching = self.find_by_id(txn, guild_id, id).await?.ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound("マッチングが見つかりません".to_string())
        })?;

        let mut active_model: quest_matchings::ActiveModel = matching.into();
        active_model.status = Set(status.to_string());
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, %id, "ステータス更新に失敗しました");
            e
        })?;

        debug!(guild_id, %id, status, "ステータスを更新しました");
        Ok(result)
    }

    async fn set_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
        recruitment_id: i32,
    ) -> Result<quest_matchings::Model> {
        debug!(guild_id, %id, recruitment_id, "募集IDを設定します");

        let matching = self.find_by_id(txn, guild_id, id).await?.ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound("マッチングが見つかりません".to_string())
        })?;

        let mut active_model: quest_matchings::ActiveModel = matching.into();
        active_model.recruitment_id = Set(Some(recruitment_id));
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, %id, "募集ID設定に失敗しました");
            e
        })?;

        debug!(guild_id, %id, recruitment_id, "募集IDを設定しました");
        Ok(result)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全てのマッチングを削除します");

        let result = quest_matchings::Entity::delete_many()
            .filter(quest_matchings::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチングの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "マッチングを削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmQuestMatchingRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmQuestMatchingRepository {
    pub fn new() -> Self {
        Self
    }
}
