use crate::models::entities::worker::notifications;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 通知リポジトリの抽象インターフェース
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    /// 指定した日時範囲内の未送信通知を取得（DatabaseConnection）
    async fn find_by_datetime_range_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>>;

    /// 指定した日時範囲内の未送信通知を取得（トランザクション内）
    async fn find_by_datetime_range_with_txn(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>>;

    /// 通知を作成（トランザクション付き）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        guild_id: i64,
        channel_id: i64,
        message_text_id: String,
    ) -> Result<notifications::Model>;

    /// task_idで通知を取得（トランザクション付き）
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<notifications::Model>>;

    /// IDで通知を取得（トランザクション付き）
    async fn find_by_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<Option<notifications::Model>>;

    /// 通知IDで通知を削除（トランザクション付き）
    async fn delete_by_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64>;

    /// すべての通知を削除（トランザクション付き）
    async fn delete_all_with_txn(&self, txn: &DatabaseTransaction) -> Result<u64>;

    /// 通知を送信済みとしてマーク（トランザクション付き）
    async fn mark_as_sent_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<notifications::Model>;
}
