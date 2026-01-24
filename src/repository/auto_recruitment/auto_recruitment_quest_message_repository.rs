//! 自動募集クエストメッセージリポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitment_quest_messages;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// 自動募集クエストメッセージリポジトリの抽象インターフェース
#[async_trait]
pub trait AutoRecruitmentQuestMessageRepository: Send + Sync {
    /// ギルドの全てのクエストメッセージを取得
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_quest_messages::Model>>;

    /// 特定のクエストのメッセージを取得
    async fn find_by_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Option<auto_recruitment_quest_messages::Model>>;

    /// クエストメッセージを作成または更新
    async fn upsert(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        message_id: i64,
    ) -> Result<auto_recruitment_quest_messages::Model>;

    /// クエストメッセージを削除
    async fn delete(&self, txn: &DatabaseTransaction, guild_id: i64, quest_id: i32) -> Result<u64>;

    /// ギルドの全てのクエストメッセージを削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;
}
