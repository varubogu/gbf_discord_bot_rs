use crate::models::entities::worker::{scheduled_task_notifications, scheduled_tasks};
use crate::repository::schedule::{NotificationWithTask, ScheduledTaskNotificationRepository};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 通知タスクリポジトリ
#[derive(Default)]
pub struct SeaOrmScheduledTaskNotificationRepository;

#[async_trait]
impl ScheduledTaskNotificationRepository for SeaOrmScheduledTaskNotificationRepository {

    /// 指定範囲内の未実行通知タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<NotificationWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行通知タスクを取得します"
        );

        // scheduled_tasks と scheduled_task_notifications を手動でJOIN
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
            .filter(scheduled_tasks::Column::TaskType.eq(1)) // Notification
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "通知タスクの取得に失敗しました");
                e
            })?;

        let mut results = Vec::new();
        for task in tasks {
            // 各タスクに対して notification_rel 情報を取得
            if let Some(notification_rel) =
                scheduled_task_notifications::Entity::find_by_id(task.id)
                    .one(txn)
                    .await
                    .map_err(|e| {
                        error!(error = %e, task_id = task.id, "通知関連情報の取得に失敗しました");
                        e
                    })?
            {
                results.push(NotificationWithTask {
                    task,
                    notification_rel,
                });
            }
        }

        debug!(count = results.len(), "未実行通知タスクを取得しました");
        Ok(results)
    }

    /// task_idで通知関連情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_notifications::Model>> {
        debug!(task_id, "通知関連情報をtask_idで取得します");

        let notification_rel = scheduled_task_notifications::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "通知関連情報の取得に失敗しました");
                e
            })?;

        debug!(
            task_id,
            found = notification_rel.is_some(),
            "通知関連情報を取得しました"
        );
        Ok(notification_rel)
    }

    /// notification_idで通知関連情報を取得
    async fn find_by_notification_id(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<Option<scheduled_task_notifications::Model>> {
        debug!(notification_id, "通知関連情報をnotification_idで取得します");

        let notification_rel = scheduled_task_notifications::Entity::find()
            .filter(scheduled_task_notifications::Column::NotificationId.eq(notification_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, notification_id, "通知関連情報の取得に失敗しました");
                e
            })?;

        debug!(
            notification_id,
            found = notification_rel.is_some(),
            "通知関連情報を取得しました"
        );
        Ok(notification_rel)
    }

    /// 通知タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        notification_id: i32,
    ) -> Result<scheduled_task_notifications::Model> {
        debug!(task_id, notification_id, "通知タスク関連情報を作成します");

        let active_model = scheduled_task_notifications::ActiveModel {
            task_id: Set(task_id),
            notification_id: Set(notification_id),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, task_id, notification_id, "通知タスク関連情報の作成に失敗しました");
            e
        })?;

        debug!(task_id, notification_id, "通知タスク関連情報を作成しました");
        Ok(model)
    }

    /// notification_idで通知タスクを削除
    async fn delete_by_notification_id(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64> {
        debug!(notification_id, "通知タスク関連情報を削除します");

        let result = scheduled_task_notifications::Entity::delete_many()
            .filter(scheduled_task_notifications::Column::NotificationId.eq(notification_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, notification_id, "通知タスク関連情報の削除に失敗しました");
                e
            })?;

        debug!(
            notification_id,
            deleted_count = result.rows_affected,
            "通知タスク関連情報を削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl SeaOrmScheduledTaskNotificationRepository {
    pub fn new() -> Self {
        Self
    }
}

