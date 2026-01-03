use crate::models::entities::guild_master::guilds;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// ギルドリポジトリの抽象インターフェース
#[async_trait]
pub trait GuildRepository: Send + Sync {
    /// ギルドを登録または更新（トランザクション内）
    /// ギルドが既に存在する場合は名前のみ更新
    async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        name: String,
    ) -> Result<guilds::Model>;
}
