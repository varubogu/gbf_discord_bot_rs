//! マッチングリポジトリの抽象インターフェース

use crate::models::entities::worker::quest_matchings;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

/// マッチングリポジトリの抽象インターフェース
#[async_trait]
pub trait QuestMatchingRepository: Send + Sync {
    /// ギルドのアクティブなマッチングを全て取得
    async fn find_active_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<quest_matchings::Model>>;

    /// 全ギルドのアクティブなマッチングを全て取得
    async fn find_all_active(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<quest_matchings::Model>>;

    /// 特定の時間・クエストのマッチングを取得
    async fn find_by_schedule(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<quest_matchings::Model>>;

    /// マッチングをIDで取得
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
    ) -> Result<Option<quest_matchings::Model>>;

    /// マッチングを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<quest_matchings::Model>;

    /// マッチングのステータスを更新
    async fn update_status(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
        status: &str,
    ) -> Result<quest_matchings::Model>;

    /// マッチングに募集IDを設定
    async fn set_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        id: Uuid,
        recruitment_id: i32,
    ) -> Result<quest_matchings::Model>;

    /// ギルドの全てのマッチングを削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;
}
