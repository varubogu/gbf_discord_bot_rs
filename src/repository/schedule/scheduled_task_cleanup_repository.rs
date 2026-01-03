use crate::models::entities::worker::{scheduled_task_cleanups, scheduled_tasks};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// クリーンアップタスクと募集の関連情報
#[derive(Debug, Clone)]
pub struct CleanupWithTask {
    pub task: scheduled_tasks::Model,
    pub cleanup: scheduled_task_cleanups::Model,
}

/// クリーンアップタスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskCleanupRepository: Send + Sync {
    /// 指定範囲内の未実行クリーンアップタスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CleanupWithTask>>;

    /// task_idでクリーンアップ情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_cleanups::Model>>;

    /// クリーンアップタスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        target_schema: String,
        target_table: String,
        cleanup_before: DateTime<Utc>,
    ) -> Result<scheduled_task_cleanups::Model>;
}
