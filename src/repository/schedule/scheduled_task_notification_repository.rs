use crate::models::entities::worker::{scheduled_task_notifications, scheduled_tasks};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 通知タスクと通知の関連情報
#[derive(Debug, Clone)]
pub struct NotificationWithTask {
    pub task: scheduled_tasks::Model,
    pub notification_rel: scheduled_task_notifications::Model,
}

/// 通知タスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskNotificationRepository: Send + Sync {
    /// 指定範囲内の未実行通知タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<NotificationWithTask>>;

    /// task_idで通知関連情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_notifications::Model>>;

    /// notification_idで通知関連情報を取得
    async fn find_by_notification_id(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<Option<scheduled_task_notifications::Model>>;

    /// 通知タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        notification_id: i32,
    ) -> Result<scheduled_task_notifications::Model>;

    /// notification_idで通知タスクを削除
    async fn delete_by_notification_id(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64>;
}
