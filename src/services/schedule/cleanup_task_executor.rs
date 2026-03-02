use crate::di::Repositories;
use crate::repository::schedule::{ScheduledTaskCleanupRepository, ScheduledTaskRepository};
use crate::services::maintenance::DataCleanupService;
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use tracing::{debug, error, info, warn};

/// データクリーンアップ実行結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupExecutionResult {
    pub deleted_recruitments: u64,
    pub deleted_notifications: u64,
    pub deleted_tasks: u64,
    pub cleanup_definition_found: bool,
}

/// task_type=3(DataCleanup) 専用Executor
///
/// - scheduled_task_cleanups の存在確認
/// - DataCleanupService 実行
/// - scheduled_tasks 実行ステータス更新
pub struct CleanupTaskExecutor {
    repos: Repositories,
}

impl CleanupTaskExecutor {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn execute(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<CleanupExecutionResult> {
        let cleanup_definition_found = match self
            .repos
            .scheduled_task_cleanup
            .find_by_task_id(txn, task_id)
            .await
        {
            Ok(Some(cleanup)) => {
                debug!(
                    task_id,
                    target_schema = %cleanup.target_schema,
                    target_table = %cleanup.target_table,
                    cleanup_before = %cleanup.cleanup_before,
                    "クリーンアップ定義を確認しました"
                );
                true
            }
            Ok(None) => {
                warn!(
                    task_id,
                    "scheduled_task_cleanupsが未登録のため、標準クリーンアップ設定で実行します"
                );
                false
            }
            Err(e) => {
                error!(task_id, error = %e, "クリーンアップ定義の取得に失敗しました");
                self.mark_task_as_failed(txn, task_id).await;
                return Err(e);
            }
        };

        let cleanup_service = DataCleanupService::new(
            self.repos.battle_recruitments,
            self.repos.notification,
            self.repos.scheduled_task,
        );

        match cleanup_service.execute(txn).await {
            Ok(stats) => {
                self.repos
                    .scheduled_task
                    .mark_as_succeeded(txn, task_id)
                    .await?;
                info!(
                    task_id,
                    deleted_recruitments = stats.deleted_recruitments,
                    deleted_notifications = stats.deleted_notifications,
                    deleted_tasks = stats.deleted_tasks,
                    cleanup_before = %stats.cleanup_before,
                    "データクリーンアップタスクを完了しました"
                );
                Ok(CleanupExecutionResult {
                    deleted_recruitments: stats.deleted_recruitments,
                    deleted_notifications: stats.deleted_notifications,
                    deleted_tasks: stats.deleted_tasks,
                    cleanup_definition_found,
                })
            }
            Err(e) => {
                error!(task_id, error = %e, "データクリーンアップの実行に失敗しました");
                self.mark_task_as_failed(txn, task_id).await;
                Err(e.into())
            }
        }
    }

    async fn mark_task_as_failed(&self, txn: &DatabaseTransaction, task_id: i32) {
        if let Err(mark_err) = self.repos.scheduled_task.mark_as_failed(txn, task_id).await {
            error!(
                task_id,
                error = %mark_err,
                "task_type=3の失敗ステータス更新に失敗しました"
            );
        }
    }
}
