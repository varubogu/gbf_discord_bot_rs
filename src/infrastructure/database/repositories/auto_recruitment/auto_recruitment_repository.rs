//! 自動募集設定リポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitments;
use crate::repository::auto_recruitment::{
    AutoRecruitmentRepository as AutoRecruitmentRepositoryTrait, CreateAutoRecruitmentParams,
};
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, Set};
use tracing::{debug, error};

/// 自動募集設定リポジトリのSeaORM実装
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentRepository;

#[async_trait]
impl AutoRecruitmentRepositoryTrait for SeaOrmAutoRecruitmentRepository {
    async fn find_all(&self, txn: &DatabaseTransaction) -> Result<Vec<auto_recruitments::Model>> {
        debug!("全ギルドの自動募集設定を取得します");

        let result = auto_recruitments::Entity::find()
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "自動募集設定の全件取得に失敗しました");
                e
            })?;

        debug!(count = result.len(), "自動募集設定を全件取得しました");
        Ok(result)
    }

    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<auto_recruitments::Model>> {
        debug!(guild_id, "ギルドIDで自動募集設定を取得します");

        let result = auto_recruitments::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "自動募集設定の取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        params: CreateAutoRecruitmentParams,
    ) -> Result<auto_recruitments::Model> {
        debug!(guild_id = params.guild_id, "自動募集設定を作成します");

        let now = chrono::Utc::now();
        let active_model = auto_recruitments::ActiveModel {
            guild_id: Set(params.guild_id),
            category_id: Set(params.category_id),
            matching_channel_id: Set(params.matching_channel_id),
            quest_channel_id: Set(params.quest_channel_id),
            matching_channel_is_bot_created: Set(params.matching_channel_is_bot_created),
            quest_channel_is_bot_created: Set(params.quest_channel_is_bot_created),
            matching_message_id: Set(params.matching_message_id),
            days_range: Set(params.days_range),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "自動募集設定の作成に失敗しました");
            e
        })?;

        debug!(guild_id = params.guild_id, "自動募集設定を作成しました");
        Ok(result)
    }

    async fn update_days_range(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        days_range: i32,
    ) -> Result<auto_recruitments::Model> {
        debug!(guild_id, days_range, "募集日数を更新します");

        let model = auto_recruitments::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "自動募集設定の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(guild_id, "自動募集設定が見つかりません");
                crate::types::AppError::Business {
                    message: format!("自動募集設定が見つかりません: {guild_id}"),
                }
            })?;

        let mut active_model: auto_recruitments::ActiveModel = model.into();
        active_model.days_range = Set(days_range);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, "募集日数の更新に失敗しました");
            e
        })?;

        debug!(guild_id, days_range, "募集日数を更新しました");
        Ok(result)
    }

    async fn update_matching_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_channel_id: Option<i64>,
    ) -> Result<auto_recruitments::Model> {
        debug!(
            guild_id,
            ?matching_channel_id,
            "マッチングチャンネルIDを更新します"
        );

        let model = auto_recruitments::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "自動募集設定の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(guild_id, "自動募集設定が見つかりません");
                crate::types::AppError::Business {
                    message: format!("自動募集設定が見つかりません: {guild_id}"),
                }
            })?;

        let mut active_model: auto_recruitments::ActiveModel = model.into();
        active_model.matching_channel_id = Set(matching_channel_id);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, "マッチングチャンネルIDの更新に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            ?matching_channel_id,
            "マッチングチャンネルIDを更新しました"
        );
        Ok(result)
    }

    async fn update_quest_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_channel_id: Option<i64>,
    ) -> Result<auto_recruitments::Model> {
        debug!(
            guild_id,
            ?quest_channel_id,
            "クエストチャンネルIDを更新します"
        );

        let model = auto_recruitments::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "自動募集設定の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(guild_id, "自動募集設定が見つかりません");
                crate::types::AppError::Business {
                    message: format!("自動募集設定が見つかりません: {guild_id}"),
                }
            })?;

        let mut active_model: auto_recruitments::ActiveModel = model.into();
        active_model.quest_channel_id = Set(quest_channel_id);
        active_model.updated_at = Set(chrono::Utc::now());

        let result = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, guild_id, "クエストチャンネルIDの更新に失敗しました");
            e
        })?;

        debug!(
            guild_id,
            ?quest_channel_id,
            "クエストチャンネルIDを更新しました"
        );
        Ok(result)
    }

    async fn delete(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64> {
        debug!(guild_id, "自動募集設定を削除します");

        let result = auto_recruitments::Entity::delete_by_id(guild_id)
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "自動募集設定の削除に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            deleted_count = result.rows_affected,
            "自動募集設定を削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmAutoRecruitmentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmAutoRecruitmentRepository {
    pub fn new() -> Self {
        Self
    }
}
