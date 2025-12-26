use crate::models::entities::worker::{scheduled_task_recurring_recruitments, scheduled_tasks};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 定期募集タスクと定期募集スケジュールの関連情報
#[derive(Debug, Clone)]
pub struct RecurringRecruitmentWithTask {
    pub task: scheduled_tasks::Model,
    pub recurring_recruitment_rel: scheduled_task_recurring_recruitments::Model,
}

/// 定期募集タスクリポジトリ
pub struct ScheduledTaskRecurringRecruitmentRepository;

impl ScheduledTaskRecurringRecruitmentRepository {
    pub fn new() -> Self {
        Self
    }

    /// 指定範囲内の未実行定期募集タスクをJOIN済みで取得
    pub async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RecurringRecruitmentWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行定期募集タスクを取得します"
        );

        // scheduled_tasks と scheduled_task_recurring_recruitments を手動でJOIN
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
            .filter(scheduled_tasks::Column::TaskType.eq(4)) // RecurringRecruitment
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "定期募集タスクの取得に失敗しました");
                e
            })?;

        let mut results = Vec::new();
        for task in tasks {
            // 各タスクに対して recurring_recruitment_rel 情報を取得
            if let Some(recurring_recruitment_rel) = scheduled_task_recurring_recruitments::Entity::find_by_id(task.id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, task_id = task.id, "定期募集関連情報の取得に失敗しました");
                    e
                })?
            {
                results.push(RecurringRecruitmentWithTask { task, recurring_recruitment_rel });
            }
        }

        debug!(count = results.len(), "未実行定期募集タスクを取得しました");
        Ok(results)
    }

    /// task_idで定期募集関連情報を取得
    pub async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>> {
        debug!(task_id, "定期募集関連情報をtask_idで取得します");

        let recurring_recruitment_rel = scheduled_task_recurring_recruitments::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "定期募集関連情報の取得に失敗しました");
                e
            })?;

        debug!(task_id, found = recurring_recruitment_rel.is_some(), "定期募集関連情報を取得しました");
        Ok(recurring_recruitment_rel)
    }

    /// schedule_idで定期募集関連情報を取得
    pub async fn find_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>> {
        debug!(schedule_id, "定期募集関連情報をschedule_idで取得します");

        let recurring_recruitment_rel = scheduled_task_recurring_recruitments::Entity::find()
            .filter(scheduled_task_recurring_recruitments::Column::ScheduleId.eq(schedule_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, schedule_id, "定期募集関連情報の取得に失敗しました");
                e
            })?;

        debug!(schedule_id, found = recurring_recruitment_rel.is_some(), "定期募集関連情報を取得しました");
        Ok(recurring_recruitment_rel)
    }

    /// 定期募集タスクを作成
    pub async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        schedule_id: i32,
    ) -> Result<scheduled_task_recurring_recruitments::Model> {
        debug!(
            task_id,
            schedule_id,
            "定期募集タスク関連情報を作成します"
        );

        let active_model = scheduled_task_recurring_recruitments::ActiveModel {
            task_id: Set(task_id),
            schedule_id: Set(schedule_id),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, task_id, schedule_id, "定期募集タスク関連情報の作成に失敗しました");
            e
        })?;

        debug!(task_id, schedule_id, "定期募集タスク関連情報を作成しました");
        Ok(model)
    }

    /// schedule_idで定期募集タスクを削除
    pub async fn delete_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<u64> {
        debug!(schedule_id, "定期募集タスク関連情報を削除します");

        let result = scheduled_task_recurring_recruitments::Entity::delete_many()
            .filter(scheduled_task_recurring_recruitments::Column::ScheduleId.eq(schedule_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, schedule_id, "定期募集タスク関連情報の削除に失敗しました");
                e
            })?;

        debug!(schedule_id, deleted_count = result.rows_affected, "定期募集タスク関連情報を削除しました");
        Ok(result.rows_affected)
    }
}
