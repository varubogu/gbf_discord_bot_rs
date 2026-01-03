use crate::models::entities::worker::{scheduled_task_dissolutions, scheduled_tasks};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 解散タスクと募集の関連情報
#[derive(Debug, Clone)]
pub struct DissolutionWithTask {
    pub task: scheduled_tasks::Model,
    pub dissolution: scheduled_task_dissolutions::Model,
}

/// 解散タスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskDissolutionRepository: Send + Sync {
    /// 指定範囲内の未実行解散タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DissolutionWithTask>>;

    /// task_idで解散情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_dissolutions::Model>>;

    /// 解散タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruit_id: i32,
    ) -> Result<scheduled_task_dissolutions::Model>;

    /// recruit_idで解散タスクを取得
    async fn find_by_recruit_id(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<Vec<scheduled_task_dissolutions::Model>>;
}
