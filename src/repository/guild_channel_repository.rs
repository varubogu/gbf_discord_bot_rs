use crate::models::entities::guild_master::guild_channels;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// ギルドチャンネルリポジトリの抽象インターフェース
#[async_trait]
pub trait GuildChannelRepository: Send + Sync {
    /// ギルドチャンネルを登録または更新（トランザクション内）
    async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
        channel_id: i64,
    ) -> Result<guild_channels::Model>;

    /// ギルドIDとチャンネル種別でギルドチャンネルを取得（トランザクション内）
    async fn get_by_guild_and_type_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
    ) -> Result<Option<guild_channels::Model>>;

    /// ギルドIDでギルドチャンネル一覧を取得（トランザクション内）
    async fn get_all_by_guild_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<guild_channels::Model>>;

    /// ギルドチャンネルを削除（トランザクション内）
    async fn delete_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
    ) -> Result<()>;
}
