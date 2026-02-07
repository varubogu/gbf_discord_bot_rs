//! 自動募集参加可能時間リポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitment_participants;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// 自動募集参加可能時間リポジトリの抽象インターフェース
#[async_trait]
pub trait AutoRecruitmentParticipantRepository: Send + Sync {
    /// ユーザーの全ての参加可能時間を取得
    async fn find_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<auto_recruitment_participants::Model>>;

    /// 特定の日時に参加可能な全ユーザーを取得
    async fn find_users_by_datetime(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Vec<auto_recruitment_participants::Model>>;

    /// 特定の日に参加可能な全ユーザーを取得
    async fn find_users_by_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
    ) -> Result<Vec<auto_recruitment_participants::Model>>;

    /// 参加可能時間を追加
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<auto_recruitment_participants::Model>;

    /// 参加可能時間を削除
    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<u64>;

    /// ユーザーの特定の日の全ての参加可能時間を削除
    async fn delete_all_by_user_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
    ) -> Result<u64>;

    /// ユーザーの全ての参加可能時間を削除
    async fn delete_all_by_user(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<u64>;

    /// ギルドの全ての参加可能時間を削除
    async fn delete_all_by_guild(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;

    /// 特定の日時の全ての参加可能時間を削除（ローテーション用）
    async fn delete_all_by_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
    ) -> Result<u64>;

    /// 全ての参加可能時間を取得
    async fn find_all(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<auto_recruitment_participants::Model>>;
}
