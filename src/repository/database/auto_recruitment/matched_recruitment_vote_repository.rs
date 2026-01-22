//! マッチング投票リポジトリのSeaORM実装

use crate::models::entities::worker::matched_recruitment_votes;
use crate::repository::auto_recruitment::MatchedRecruitmentVoteRepository as MatchedRecruitmentVoteRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// マッチング投票リポジトリのSeaORM実装
pub struct SeaOrmMatchedRecruitmentVoteRepository;

#[async_trait]
impl MatchedRecruitmentVoteRepositoryTrait for SeaOrmMatchedRecruitmentVoteRepository {
    async fn find_by_matched_channel_id(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
    ) -> Result<Vec<matched_recruitment_votes::Model>> {
        debug!(matched_channel_id, "マッチング済み募集の全投票を取得します");

        let result = matched_recruitment_votes::Entity::find()
            .filter(matched_recruitment_votes::Column::MatchedChannelId.eq(matched_channel_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, matched_channel_id, "投票の取得に失敗しました");
                e
            })?;

        debug!(
            matched_channel_id,
            count = result.len(),
            "投票を取得しました"
        );
        Ok(result)
    }

    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
    ) -> Result<Option<matched_recruitment_votes::Model>> {
        debug!(matched_channel_id, user_id, "ユーザーの投票を取得します");

        let result = matched_recruitment_votes::Entity::find()
            .filter(matched_recruitment_votes::Column::MatchedChannelId.eq(matched_channel_id))
            .filter(matched_recruitment_votes::Column::UserId.eq(user_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, matched_channel_id, user_id, "投票の取得に失敗しました");
                e
            })?;

        Ok(result)
    }

    async fn upsert(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
        quest_id: Option<i32>,
    ) -> Result<matched_recruitment_votes::Model> {
        debug!(
            matched_channel_id,
            user_id,
            ?quest_id,
            "投票を作成または更新します"
        );

        // 既存の投票を検索
        let existing = matched_recruitment_votes::Entity::find()
            .filter(matched_recruitment_votes::Column::MatchedChannelId.eq(matched_channel_id))
            .filter(matched_recruitment_votes::Column::UserId.eq(user_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, matched_channel_id, user_id, "投票の検索に失敗しました");
                e
            })?;

        let now = chrono::Utc::now();

        if let Some(model) = existing {
            // 更新
            let mut active_model: matched_recruitment_votes::ActiveModel = model.into();
            active_model.quest_id = Set(quest_id);
            active_model.updated_at = Set(now);

            let result = active_model.update(txn).await.map_err(|e| {
                error!(error = %e, matched_channel_id, user_id, "投票の更新に失敗しました");
                e
            })?;

            debug!(matched_channel_id, user_id, ?quest_id, "投票を更新しました");
            Ok(result)
        } else {
            // 新規作成
            let active_model = matched_recruitment_votes::ActiveModel {
                id: sea_orm::NotSet,
                matched_channel_id: Set(matched_channel_id),
                user_id: Set(user_id),
                quest_id: Set(quest_id),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let result = active_model.insert(txn).await.map_err(|e| {
                error!(error = %e, matched_channel_id, user_id, "投票の作成に失敗しました");
                e
            })?;

            debug!(matched_channel_id, user_id, ?quest_id, "投票を作成しました");
            Ok(result)
        }
    }

    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
    ) -> Result<u64> {
        debug!(matched_channel_id, user_id, "投票を削除します");

        let result = matched_recruitment_votes::Entity::delete_many()
            .filter(matched_recruitment_votes::Column::MatchedChannelId.eq(matched_channel_id))
            .filter(matched_recruitment_votes::Column::UserId.eq(user_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, matched_channel_id, user_id, "投票の削除に失敗しました");
                e
            })?;

        debug!(
            matched_channel_id,
            user_id,
            deleted_count = result.rows_affected,
            "投票を削除しました"
        );
        Ok(result.rows_affected)
    }

    async fn delete_all_by_matched_channel_id(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
    ) -> Result<u64> {
        debug!(matched_channel_id, "マッチング済み募集の全投票を削除します");

        let result = matched_recruitment_votes::Entity::delete_many()
            .filter(matched_recruitment_votes::Column::MatchedChannelId.eq(matched_channel_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, matched_channel_id, "投票の削除に失敗しました");
                e
            })?;

        debug!(
            matched_channel_id,
            deleted_count = result.rows_affected,
            "投票を削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl Default for SeaOrmMatchedRecruitmentVoteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmMatchedRecruitmentVoteRepository {
    pub fn new() -> Self {
        Self
    }
}
