//! マッチングユーザーリポジトリの抽象インターフェース

use crate::models::entities::worker::quest_matching_users;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

/// マッチングユーザーリポジトリの抽象インターフェース
#[async_trait]
pub trait QuestMatchingUserRepository: Send + Sync {
    /// マッチングの参加中ユーザーを全て取得
    async fn find_active_by_matching(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
    ) -> Result<Vec<quest_matching_users::Model>>;

    /// ユーザーが参加中のマッチングを全て取得
    async fn find_active_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<quest_matching_users::Model>>;

    /// マッチングユーザーを追加
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
        battle_style_id: Option<i32>,
    ) -> Result<quest_matching_users::Model>;

    /// マッチングユーザーの属性を更新
    async fn update_battle_style(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
        battle_style_id: Option<i32>,
    ) -> Result<quest_matching_users::Model>;

    /// マッチングユーザーを離脱させる（left_atを設定）
    async fn leave(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        user_id: i64,
    ) -> Result<quest_matching_users::Model>;

    /// ギルドの全てのマッチングユーザーを削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;
}
