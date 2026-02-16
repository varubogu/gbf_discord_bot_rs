use crate::models::entities::worker::scheduled_tasks::{self, TaskExecutionStatus};
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

    /// タスクを正常終了にマーク
    async fn mark_as_succeeded(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model>;

    /// タスクを警告付き正常終了にマーク
    async fn mark_as_succeeded_with_warning(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model>;

    /// タスクを異常終了にマーク
    async fn mark_as_failed(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model>;

    /// タスクの実行状態を更新
    async fn update_execution_status(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        status: TaskExecutionStatus,
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

    /// 複数IDでタスクを取得（N+1問題解消用、トランザクション対応）
    async fn find_many_by_ids_with_txn(
        &self,
        txn: &DatabaseTransaction,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>>;

    /// 複数IDでタスクを取得（N+1問題解消用、DB接続対応）
    async fn find_many_by_ids_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>>;

    /// 指定日時より前のタスクを削除（クリーンアップ用）
    async fn delete_before_date_with_txn(
        &self,
        txn: &DatabaseTransaction,
        before: DateTime<Utc>,
    ) -> Result<u64>;
}

/// Arc<T>に対するScheduledTaskRepositoryの実装
/// これによりArc<ConcreteRepository>を直接使用できる
#[async_trait]
impl<T> ScheduledTaskRepository for std::sync::Arc<T>
where
    T: ScheduledTaskRepository + ?Sized,
{
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        (**self).find_pending_in_range(txn, from, to).await
    }

    async fn find_pending_to(
        &self,
        txn: &DatabaseTransaction,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        (**self).find_pending_to(txn, to).await
    }

    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_tasks::Model>> {
        (**self).find_by_id(txn, task_id).await
    }

    async fn create(
        &self,
        txn: &DatabaseTransaction,
        schedule_datetime: DateTime<Utc>,
        task_type: i32,
        guild_id: Option<i64>,
        channel_id: Option<i64>,
    ) -> Result<scheduled_tasks::Model> {
        (**self)
            .create(txn, schedule_datetime, task_type, guild_id, channel_id)
            .await
    }

    async fn mark_as_succeeded(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        (**self).mark_as_succeeded(txn, task_id).await
    }

    async fn mark_as_succeeded_with_warning(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        (**self).mark_as_succeeded_with_warning(txn, task_id).await
    }

    async fn mark_as_failed(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        (**self).mark_as_failed(txn, task_id).await
    }

    async fn update_execution_status(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        status: TaskExecutionStatus,
    ) -> Result<scheduled_tasks::Model> {
        (**self).update_execution_status(txn, task_id, status).await
    }

    async fn delete_by_id(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<u64> {
        (**self).delete_by_id(txn, task_id).await
    }

    async fn delete_dissolutions_by_recruit_id(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<u64> {
        (**self)
            .delete_dissolutions_by_recruit_id(txn, recruit_id)
            .await
    }

    async fn delete_all_by_task_type(
        &self,
        txn: &DatabaseTransaction,
        task_type: i32,
    ) -> Result<u64> {
        (**self).delete_all_by_task_type(txn, task_type).await
    }

    async fn find_many_by_ids_with_txn(
        &self,
        txn: &DatabaseTransaction,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        (**self).find_many_by_ids_with_txn(txn, ids).await
    }

    async fn find_many_by_ids_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        (**self).find_many_by_ids_with_db(db, ids).await
    }

    async fn delete_before_date_with_txn(
        &self,
        txn: &DatabaseTransaction,
        before: DateTime<Utc>,
    ) -> Result<u64> {
        (**self).delete_before_date_with_txn(txn, before).await
    }
}
