//! マッチング投票リポジトリの抽象インターフェース

use crate::models::entities::worker::matched_recruitment_votes;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// マッチング投票リポジトリの抽象インターフェース
#[async_trait]
pub trait MatchedRecruitmentVoteRepository: Send + Sync {
    /// マッチング済み募集の全ての投票を取得
    async fn find_by_matched_channel_id(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
    ) -> Result<Vec<matched_recruitment_votes::Model>>;

    /// ユーザーの投票を取得
    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
    ) -> Result<Option<matched_recruitment_votes::Model>>;

    /// 投票を作成または更新（upsert）
    async fn upsert(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
        quest_id: Option<i32>,
    ) -> Result<matched_recruitment_votes::Model>;

    /// 投票を削除
    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
    ) -> Result<u64>;

    /// マッチング済み募集の全ての投票を削除
    async fn delete_all_by_matched_channel_id(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
    ) -> Result<u64>;
}
