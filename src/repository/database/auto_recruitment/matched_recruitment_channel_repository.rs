//! マッチング済み募集チャンネルリポジトリのSeaORM実装

use crate::models::entities::worker::matched_recruitment_channels;
use crate::repository::auto_recruitment::MatchedRecruitmentChannelRepository as MatchedRecruitmentChannelRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// マッチング済み募集チャンネルリポジトリのSeaORM実装
pub struct SeaOrmMatchedRecruitmentChannelRepository;

#[async_trait]
impl MatchedRecruitmentChannelRepositoryTrait for SeaOrmMatchedRecruitmentChannelRepository {
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
    ) -> Result<Option<matched_recruitment_channels::Model>> {
        debug!(id, "IDでマッチング済み募集を取得します");

        let result = matched_recruitment_channels::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "マッチング済み募集の取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<matched_recruitment_channels::Model>> {
        debug!(guild_id, "ギルドIDでマッチング済み募集を取得します");

        let result = matched_recruitment_channels::Entity::find()
            .filter(matched_recruitment_channels::Column::GuildId.eq(guild_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチング済み募集の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "マッチング済み募集を取得しました"
        );
        Ok(result)
    }

    async fn find_by_datetime(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<matched_recruitment_channels::Model>> {
        debug!(
            guild_id,
            month, day, hour, "日時でマッチング済み募集を取得します"
        );

        let result = matched_recruitment_channels::Entity::find()
            .filter(matched_recruitment_channels::Column::GuildId.eq(guild_id))
            .filter(matched_recruitment_channels::Column::Month.eq(month))
            .filter(matched_recruitment_channels::Column::Day.eq(day))
            .filter(matched_recruitment_channels::Column::Hour.eq(hour))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, month, day, hour, "マッチング済み募集の取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn find_undecided(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<matched_recruitment_channels::Model>> {
        debug!(guild_id, "未決定のマッチング済み募集を取得します");

        let result = matched_recruitment_channels::Entity::find()
            .filter(matched_recruitment_channels::Column::GuildId.eq(guild_id))
            .filter(matched_recruitment_channels::Column::IsDecided.eq(false))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチング済み募集の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "未決定のマッチング済み募集を取得しました"
        );
        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<matched_recruitment_channels::Model> {
        debug!(
            guild_id,
            channel_id, message_id, month, day, hour, "マッチング済み募集を作成します"
        );

        let now = chrono::Utc::now();
        let active_model = matched_recruitment_channels::ActiveModel {
            id: sea_orm::NotSet,
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            month: Set(month),
            day: Set(day),
            hour: Set(hour),
            quest_id: Set(None),
            is_decided: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, guild_id, channel_id, "マッチング済み募集の作成に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            channel_id,
            id = result.id,
            "マッチング済み募集を作成しました"
        );
        Ok(result)
    }

    async fn update_message_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        message_id: i64,
    ) -> Result<matched_recruitment_channels::Model> {
        debug!(id, message_id, "メッセージIDを更新します");

        let model = matched_recruitment_channels::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "マッチング済み募集の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(id, "マッチング済み募集が見つかりません");
                crate::types::AppError::Business {
                    message: format!("マッチング済み募集が見つかりません: {id}"),
                }
            })?;

        let mut active_model: matched_recruitment_channels::ActiveModel = model.into();
        active_model.message_id = Set(message_id);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, id, "メッセージIDの更新に失敗しました");
            e
        })?;

        debug!(id, message_id, "メッセージIDを更新しました");
        Ok(result)
    }

    async fn decide_quest(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        quest_id: i32,
    ) -> Result<matched_recruitment_channels::Model> {
        debug!(id, quest_id, "クエストを決定します");

        let model = matched_recruitment_channels::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "マッチング済み募集の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(id, "マッチング済み募集が見つかりません");
                crate::types::AppError::Business {
                    message: format!("マッチング済み募集が見つかりません: {id}"),
                }
            })?;

        let mut active_model: matched_recruitment_channels::ActiveModel = model.into();
        active_model.quest_id = Set(Some(quest_id));
        active_model.is_decided = Set(true);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, id, "クエスト決定の更新に失敗しました");
            e
        })?;

        debug!(id, quest_id, "クエストを決定しました");
        Ok(result)
    }

    async fn delete(&self, txn: &DatabaseTransaction, id: i32) -> Result<u64> {
        debug!(id, "マッチング済み募集を削除します");

        let result = matched_recruitment_channels::Entity::delete_by_id(id)
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "マッチング済み募集の削除に失敗しました");
                e
            })?;

        debug!(
            id,
            deleted_count = result.rows_affected,
            "マッチング済み募集を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "ギルドの全てのマッチング済み募集を削除します");

        let result = matched_recruitment_channels::Entity::delete_many()
            .filter(matched_recruitment_channels::Column::GuildId.eq(guild_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチング済み募集の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "マッチング済み募集を削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmMatchedRecruitmentChannelRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmMatchedRecruitmentChannelRepository {
    pub fn new() -> Self {
        Self
    }
}
