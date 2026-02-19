//! 自動募集日時チャンネルリポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitment_channels;
use crate::repository::auto_recruitment::AutoRecruitmentChannelRepository as AutoRecruitmentChannelRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tracing::{debug, error};

/// 自動募集日時チャンネルリポジトリのSeaORM実装
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentChannelRepository;

#[async_trait]
impl AutoRecruitmentChannelRepositoryTrait for SeaOrmAutoRecruitmentChannelRepository {
    async fn find_all(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<auto_recruitment_channels::Model>> {
        debug!("全ての日時チャンネルを取得します");

        let result = auto_recruitment_channels::Entity::find()
            .order_by_asc(auto_recruitment_channels::Column::GuildId)
            .order_by_asc(auto_recruitment_channels::Column::SortOrder)
            .order_by_asc(auto_recruitment_channels::Column::Month)
            .order_by_asc(auto_recruitment_channels::Column::Day)
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "日時チャンネルの取得に失敗しました");
                e
            })?;

        debug!(count = result.len(), "日時チャンネルを取得しました");
        Ok(result)
    }

    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_channels::Model>> {
        debug!(guild_id, "ギルドIDで日時チャンネルを取得します");

        let result = auto_recruitment_channels::Entity::find()
            .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
            .order_by_asc(auto_recruitment_channels::Column::SortOrder)
            .order_by_asc(auto_recruitment_channels::Column::Month)
            .order_by_asc(auto_recruitment_channels::Column::Day)
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "日時チャンネルの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "日時チャンネルを取得しました"
        );
        Ok(result)
    }

    async fn find_by_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<Option<auto_recruitment_channels::Model>> {
        debug!(
            guild_id,
            channel_id, "チャンネルIDで日時チャンネルを取得します"
        );

        let result = auto_recruitment_channels::Entity::find()
            .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_channels::Column::ChannelId.eq(channel_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, channel_id, "日時チャンネルの取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        month: i32,
        day: i32,
        sort_order: i32,
        is_bot_created: bool,
        message_id: Option<i64>,
    ) -> Result<auto_recruitment_channels::Model> {
        debug!(
            guild_id,
            channel_id, month, day, is_bot_created, "日時チャンネルを作成します"
        );

        let now = chrono::Utc::now();
        let active_model = auto_recruitment_channels::ActiveModel {
            id: sea_orm::NotSet,
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            month: Set(month),
            day: Set(day),
            sort_order: Set(sort_order),
            is_bot_created: Set(is_bot_created),
            message_id: Set(message_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, channel_id, "日時チャンネルの作成に失敗しました");
            e
        })?;

        debug!(guild_id, channel_id, "日時チャンネルを作成しました");
        Ok(result)
    }

    async fn update_date(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        month: i32,
        day: i32,
    ) -> Result<auto_recruitment_channels::Model> {
        debug!(id, month, day, "日時チャンネルの日付を更新します");

        let model = auto_recruitment_channels::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "日時チャンネルの取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(id, "日時チャンネルが見つかりません");
                crate::types::AppError::Business {
                    message: format!("日時チャンネルが見つかりません: {id}"),
                }
            })?;

        let mut active_model: auto_recruitment_channels::ActiveModel = model.into();
        active_model.month = Set(month);
        active_model.day = Set(day);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, id, "日時チャンネルの日付更新に失敗しました");
            e
        })?;

        debug!(id, month, day, "日時チャンネルの日付を更新しました");
        Ok(result)
    }

    async fn delete_by_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<u64> {
        debug!(guild_id, channel_id, "日時チャンネルを削除します");

        let result = auto_recruitment_channels::Entity::delete_many()
            .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_channels::Column::ChannelId.eq(channel_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, channel_id, "日時チャンネルの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            channel_id,
            deleted_count = result.rows_affected,
            "日時チャンネルを削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<u64> {
        debug!(guild_id, "ギルドの全ての日時チャンネルを削除します");

        let result = auto_recruitment_channels::Entity::delete_many()
            .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "日時チャンネルの削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "日時チャンネルを削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmAutoRecruitmentChannelRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmAutoRecruitmentChannelRepository {
    pub fn new() -> Self {
        Self
    }
}
