//! マッチング済み募集チャンネルリポジトリの抽象インターフェース

use crate::models::entities::worker::matched_recruitment_channels;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// マッチング済み募集チャンネルリポジトリの抽象インターフェース
#[async_trait]
pub trait MatchedRecruitmentChannelRepository: Send + Sync {
    /// IDでマッチング済み募集を取得
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
    ) -> Result<Option<matched_recruitment_channels::Model>>;

    /// ギルドIDで全てのマッチング済み募集を取得
    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<matched_recruitment_channels::Model>>;

    /// 特定の日時のマッチング済み募集を取得
    async fn find_by_datetime(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<matched_recruitment_channels::Model>>;

    /// 未決定のマッチング済み募集を取得
    async fn find_undecided(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<matched_recruitment_channels::Model>>;

    /// マッチング済み募集を作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<matched_recruitment_channels::Model>;

    /// メッセージIDを更新（参加者追加時のメッセージ編集後）
    async fn update_message_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        message_id: i64,
    ) -> Result<matched_recruitment_channels::Model>;

    /// クエストを決定
    async fn decide_quest(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        quest_id: i32,
    ) -> Result<matched_recruitment_channels::Model>;

    /// マッチング済み募集を削除
    async fn delete(&self, txn: &DatabaseTransaction, id: i32) -> Result<u64>;

    /// ギルドの全てのマッチング済み募集を削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;
}
