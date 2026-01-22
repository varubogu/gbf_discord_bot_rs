//! 自動募集日時チャンネルリポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitment_channels;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// 自動募集日時チャンネルリポジトリの抽象インターフェース
#[async_trait]
pub trait AutoRecruitmentChannelRepository: Send + Sync {
    /// 全ての日時チャンネルを取得
    async fn find_all(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<auto_recruitment_channels::Model>>;

    /// ギルドIDで全ての日時チャンネルを取得（日付昇順）
    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_channels::Model>>;

    /// チャンネルIDで日時チャンネルを取得
    async fn find_by_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<Option<auto_recruitment_channels::Model>>;

    /// 日時チャンネルを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        month: i32,
        day: i32,
        sort_order: i32,
    ) -> Result<auto_recruitment_channels::Model>;

    /// 日時チャンネルの日付を更新
    async fn update_date(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        month: i32,
        day: i32,
    ) -> Result<auto_recruitment_channels::Model>;

    /// チャンネルIDで日時チャンネルを削除
    async fn delete_by_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<u64>;

    /// ギルドIDで全ての日時チャンネルを削除
    async fn delete_all_by_guild_id(&self, txn: &DatabaseTransaction, guild_id: i64)
    -> Result<u64>;
}
