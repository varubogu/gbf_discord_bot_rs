//! 自動募集参加可能時間リポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitment_participants;
use crate::repository::auto_recruitment::AutoRecruitmentParticipantRepository as AutoRecruitmentParticipantRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 自動募集参加可能時間リポジトリのSeaORM実装
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentParticipantRepository;

#[async_trait]
impl AutoRecruitmentParticipantRepositoryTrait for SeaOrmAutoRecruitmentParticipantRepository {
    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<auto_recruitment_participants::Model>> {
        debug!(guild_id, user_id, "ユーザーの参加可能時間を取得します");

        let result = auto_recruitment_participants::Entity::find()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::UserId.eq(user_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, "参加可能時間の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            count = result.len(),
            "参加可能時間を取得しました"
        );
        Ok(result)
    }

    async fn find_users_by_datetime(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Vec<auto_recruitment_participants::Model>> {
        debug!(
            guild_id,
            month, day, hour, "指定日時に参加可能なユーザーを取得します"
        );

        let result = auto_recruitment_participants::Entity::find()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::Month.eq(month))
            .filter(auto_recruitment_participants::Column::Day.eq(day))
            .filter(auto_recruitment_participants::Column::Hour.eq(hour))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, month, day, hour, "参加可能時間の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            month,
            day,
            hour,
            count = result.len(),
            "参加可能なユーザーを取得しました"
        );
        Ok(result)
    }

    async fn find_users_by_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
    ) -> Result<Vec<auto_recruitment_participants::Model>> {
        debug!(
            guild_id,
            month, day, "指定日に参加可能なユーザーを取得します"
        );

        let result = auto_recruitment_participants::Entity::find()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::Month.eq(month))
            .filter(auto_recruitment_participants::Column::Day.eq(day))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, month, day, "参加可能時間の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            month,
            day,
            count = result.len(),
            "参加可能なユーザーを取得しました"
        );
        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<auto_recruitment_participants::Model> {
        debug!(
            guild_id,
            user_id, month, day, hour, "参加可能時間を追加します"
        );

        let now = chrono::Utc::now();
        let active_model = auto_recruitment_participants::ActiveModel {
            guild_id: Set(guild_id),
            user_id: Set(user_id),
            month: Set(month),
            day: Set(day),
            hour: Set(hour),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, user_id, month, day, hour, "参加可能時間の追加に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            user_id, month, day, hour, "参加可能時間を追加しました"
        );
        Ok(result)
    }

    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<u64> {
        debug!(
            guild_id,
            user_id, month, day, hour, "参加可能時間を削除します"
        );

        let result = auto_recruitment_participants::Entity::delete_many()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::UserId.eq(user_id))
            .filter(auto_recruitment_participants::Column::Month.eq(month))
            .filter(auto_recruitment_participants::Column::Day.eq(day))
            .filter(auto_recruitment_participants::Column::Hour.eq(hour))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, month, day, hour, "参加可能時間の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            month,
            day,
            hour,
            deleted_count = result.rows_affected,
            "参加可能時間を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_user_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
    ) -> Result<u64> {
        debug!(
            guild_id,
            user_id, month, day, "ユーザーの指定日の参加可能時間を全て削除します"
        );

        let result = auto_recruitment_participants::Entity::delete_many()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::UserId.eq(user_id))
            .filter(auto_recruitment_participants::Column::Month.eq(month))
            .filter(auto_recruitment_participants::Column::Day.eq(day))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, month, day, "参加可能時間の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            month,
            day,
            deleted_count = result.rows_affected,
            "参加可能時間を削除しました"
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
            user_id, "ユーザーの全ての参加可能時間を削除します"
        );

        let result = auto_recruitment_participants::Entity::delete_many()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::UserId.eq(user_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, user_id, "参加可能時間の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            user_id,
            deleted_count = result.rows_affected,
            "参加可能時間を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全ての参加可能時間を削除します");

        let result = auto_recruitment_participants::Entity::delete_many()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "参加可能時間の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "参加可能時間を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
    ) -> Result<u64> {
        debug!(
            guild_id,
            month, day, "指定日の全ての参加可能時間を削除します"
        );

        let result = auto_recruitment_participants::Entity::delete_many()
            .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_participants::Column::Month.eq(month))
            .filter(auto_recruitment_participants::Column::Day.eq(day))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, month, day, "参加可能時間の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            month,
            day,
            deleted_count = result.rows_affected,
            "参加可能時間を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn find_all(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<auto_recruitment_participants::Model>> {
        debug!("全ての参加可能時間を取得します");

        let result = auto_recruitment_participants::Entity::find()
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "参加可能時間の取得に失敗しました");
                e
            })?;

        debug!(count = result.len(), "参加可能時間を取得しました");
        Ok(result)
    }
}

impl Default for SeaOrmAutoRecruitmentParticipantRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmAutoRecruitmentParticipantRepository {
    pub fn new() -> Self {
        Self
    }
}
