use crate::models::entities::worker::{scheduled_task_cleanups, scheduled_tasks};
use crate::repository::schedule::{CleanupWithTask, ScheduledTaskCleanupRepository};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// クリーンアップタスクリポジトリ
pub struct SeaOrmScheduledTaskCleanupRepository;

#[async_trait]
impl ScheduledTaskCleanupRepository for SeaOrmScheduledTaskCleanupRepository {
    /// 指定範囲内の未実行クリーンアップタスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CleanupWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行クリーンアップタスクを取得します"
        );

        // scheduled_tasks と scheduled_task_cleanups を手動でJOIN
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
            .filter(scheduled_tasks::Column::TaskType.eq(3)) // DataCleanup
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "クリーンアップタスクの取得に失敗しました");
                e
            })?;

        let mut results = Vec::new();
        for task in tasks {
            // 各タスクに対して cleanup 情報を取得
            if let Some(cleanup) = scheduled_task_cleanups::Entity::find_by_id(task.id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, task_id = task.id, "クリーンアップ情報の取得に失敗しました");
                    e
                })?
            {
                results.push(CleanupWithTask { task, cleanup });
            }
        }

        debug!(
            count = results.len(),
            "未実行クリーンアップタスクを取得しました"
        );
        Ok(results)
    }

    /// task_idでクリーンアップ情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_cleanups::Model>> {
        debug!(task_id, "クリーンアップ情報をtask_idで取得します");

        let cleanup = scheduled_task_cleanups::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "クリーンアップ情報の取得に失敗しました");
                e
            })?;

        Ok(cleanup)
    }

    /// クリーンアップタスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        target_schema: String,
        target_table: String,
        cleanup_before: DateTime<Utc>,
    ) -> Result<scheduled_task_cleanups::Model> {
        debug!(
            task_id,
            target_schema,
            target_table,
            cleanup_before = %cleanup_before,
            "クリーンアップタスクを作成します"
        );

        let active_model = scheduled_task_cleanups::ActiveModel {
            task_id: Set(task_id),
            target_schema: Set(target_schema.clone()),
            target_table: Set(target_table.clone()),
            cleanup_before: Set(cleanup_before),
        };

        let cleanup = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "クリーンアップタスクの作成に失敗しました");
            e
        })?;

        debug!(
            task_id,
            target_schema, target_table, "クリーンアップタスクを作成しました"
        );
        Ok(cleanup)
    }
}

impl SeaOrmScheduledTaskCleanupRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmScheduledTaskCleanupRepository {
    fn default() -> Self {
        Self::new()
    }
}
