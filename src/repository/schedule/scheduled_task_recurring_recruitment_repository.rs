use crate::models::entities::worker::scheduled_task_recurring_recruitments;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 定期募集タスクと定期募集スケジュールの関連情報
#[derive(Debug, Clone)]
pub struct RecurringRecruitmentWithTask {
    pub task: crate::models::entities::worker::scheduled_tasks::Model,
    pub recurring_recruitment_rel: scheduled_task_recurring_recruitments::Model,
}

/// 定期募集タスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskRecurringRecruitmentRepository: Send + Sync {
    /// 指定範囲内の未実行定期募集タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RecurringRecruitmentWithTask>>;

    /// scheduled_task_idで定期募集関連情報を取得
    async fn find_by_scheduled_task_id(
        &self,
        txn: &DatabaseTransaction,
        scheduled_task_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>>;

    /// recruitment_schedule_idで定期募集関連情報を取得
    async fn find_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>>;

    /// 定期募集タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        scheduled_task_id: i32,
        recruitment_schedule_id: i32,
    ) -> Result<scheduled_task_recurring_recruitments::Model>;

    /// recruitment_schedule_idで定期募集タスクを削除
    async fn delete_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<u64>;

    /// recruitment_schedule_idに紐づく未実行のscheduled_tasksを削除
    async fn delete_pending_tasks_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<u64>;
}
