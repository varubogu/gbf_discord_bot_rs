use crate::di::Repositories;
use crate::models::entities::worker::{notifications, scheduled_tasks};
use crate::repository::schedule::{NotificationRepository, ScheduledTaskRepository};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;
use tracing::error;

/// タスクディスパッチが必要とする scheduled_tasks / notifications への
/// 直接アクセスを集約する薄いサービス。
///
/// facade層がrepositoryを直接呼ばずに済むよう、ディスパッチ処理専用の窓口として存在する。
pub struct ScheduledTaskDispatchSupportService {
    repos: Repositories,
}

impl ScheduledTaskDispatchSupportService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    /// 指定日時までに実行期限を迎える未実行タスクを取得
    pub async fn find_due_tasks(
        &self,
        txn: &DatabaseTransaction,
        until: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        self.repos.scheduled_task.find_pending_to(txn, until).await
    }

    /// task_idに紐づく通知を取得（通知タスク実行用）
    pub async fn find_notification_for_task(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<notifications::Model>> {
        self.repos.notification.find_by_task_id(txn, task_id).await
    }

    /// タスクを正常終了にマーク
    pub async fn mark_succeeded(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
        self.repos
            .scheduled_task
            .mark_as_succeeded(txn, task_id)
            .await?;
        Ok(())
    }

    /// タスクを警告付き正常終了にマーク
    pub async fn mark_succeeded_with_warning(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<()> {
        self.repos
            .scheduled_task
            .mark_as_succeeded_with_warning(txn, task_id)
            .await?;
        Ok(())
    }

    /// タスクを異常終了にマーク（マーク自体に失敗してもログのみで握りつぶす）
    pub async fn mark_failed(&self, txn: &DatabaseTransaction, task_id: i32) {
        if let Err(e) = self.repos.scheduled_task.mark_as_failed(txn, task_id).await {
            error!(task_id, error = %e, "タスクの失敗マークに失敗しました");
        }
    }
}
