//! ユーザー希望クエストリポジトリの抽象インターフェース

use crate::models::entities::guild_master::user_desired_quests;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// ユーザー希望クエストリポジトリの抽象インターフェース
#[async_trait]
pub trait UserDesiredQuestRepository: Send + Sync {
    /// ユーザーの全ての希望クエストを取得
    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<user_desired_quests::Model>>;

    /// 特定のクエストを希望している全ユーザーを取得
    async fn find_users_by_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Vec<user_desired_quests::Model>>;

    /// 複数のクエストを希望している全ユーザーを取得
    async fn find_users_by_quests(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_ids: Vec<i32>,
    ) -> Result<Vec<user_desired_quests::Model>>;

    /// 希望クエストを追加
    /// battle_style_id: 0なら属性指定なし、1-6なら各属性
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
        battle_style_id: i32,
    ) -> Result<user_desired_quests::Model>;

    /// 希望クエストを削除（特定の属性のみ）
    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
        battle_style_id: i32,
    ) -> Result<u64>;

    /// 希望クエストを全属性削除
    async fn delete_all_styles(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
    ) -> Result<u64>;

    /// ユーザーの全ての希望クエストを削除
    async fn delete_all_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<u64>;

    /// ギルドの全ての希望クエストを削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;

    /// 全ての希望クエストを取得
    async fn find_all(&self, txn: &DatabaseTransaction) -> Result<Vec<user_desired_quests::Model>>;
}
