use crate::models::entities::worker::scheduled_tasks;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// スケジュールタスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskRepository: Send + Sync {
    /// 指定した日時範囲内の未実行タスクを取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>>;

    /// 指定した日時以前の未実行タスクを取得
    async fn find_pending_to(
        &self,
        txn: &DatabaseTransaction,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>>;

    /// IDでタスクを取得（DB再確認用）
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_tasks::Model>>;

    /// タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        schedule_datetime: DateTime<Utc>,
        task_type: i32,
        guild_id: Option<i64>,
        channel_id: Option<i64>,
    ) -> Result<scheduled_tasks::Model>;

    /// タスクを実行済みにマーク
    async fn mark_as_executed(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model>;

    /// IDでタスクを削除
    async fn delete_by_id(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<u64>;

    /// recruit_idに紐づく解散タスクを削除
    async fn delete_dissolutions_by_recruit_id(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<u64>;

    /// 指定したtask_typeのタスクを全て削除
    async fn delete_all_by_task_type(
        &self,
        txn: &DatabaseTransaction,
        task_type: i32,
    ) -> Result<u64>;
}
