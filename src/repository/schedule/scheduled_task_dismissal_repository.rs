use crate::models::entities::worker::{scheduled_task_dismissals, scheduled_tasks};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 解散タスクと解散設定の関連情報
#[derive(Debug, Clone)]
pub struct DismissalWithTask {
    pub task: scheduled_tasks::Model,
    pub dismissal_rel: scheduled_task_dismissals::Model,
}

/// 解散タスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskDismissalRepository: Send + Sync {
    /// 指定範囲内の未実行解散タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DismissalWithTask>>;

    /// task_idで解散関連情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_dismissals::Model>>;

    /// recruitment_dismissal_idで解散関連情報を取得
    async fn find_by_recruitment_dismissal_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_dismissal_id: i32,
    ) -> Result<Option<scheduled_task_dismissals::Model>>;

    /// 解散タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruitment_dismissal_id: i32,
    ) -> Result<scheduled_task_dismissals::Model>;

    /// recruitment_dismissal_idで解散タスクを削除
    async fn delete_by_recruitment_dismissal_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_dismissal_id: i32,
    ) -> Result<u64>;
}
