use crate::models::entities::worker::{scheduled_task_recurring_recruitments, scheduled_tasks};
use crate::repository::schedule::{
    RecurringRecruitmentWithTask,
    ScheduledTaskRecurringRecruitmentRepository as ScheduledTaskRecurringRecruitmentRepositoryTrait,
    ScheduledTaskRepository,
};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 定期募集タスクリポジトリ
#[derive(Default)]
pub struct SeaOrmScheduledTaskRecurringRecruitmentRepository;

#[async_trait]
impl ScheduledTaskRecurringRecruitmentRepositoryTrait
    for SeaOrmScheduledTaskRecurringRecruitmentRepository
{
    /// 指定範囲内の未実行定期募集タスクをJOIN済みで取得
    async fn find_pending_in_range(
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

    /// scheduled_task_idで定期募集関連情報を取得
    async fn find_by_scheduled_task_id(
        &self,
        txn: &DatabaseTransaction,
        scheduled_task_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>> {
        debug!(
            scheduled_task_id,
            "定期募集関連情報をscheduled_task_idで取得します"
        );

        let recurring_recruitment_rel =
            scheduled_task_recurring_recruitments::Entity::find_by_id(scheduled_task_id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, scheduled_task_id, "定期募集関連情報の取得に失敗しました");
                    e
                })?;

        debug!(
            scheduled_task_id,
            found = recurring_recruitment_rel.is_some(),
            "定期募集関連情報を取得しました"
        );
        Ok(recurring_recruitment_rel)
    }

    /// recruitment_schedule_idで定期募集関連情報を取得
    async fn find_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>> {
        debug!(
            recruitment_schedule_id,
            "定期募集関連情報をrecruitment_schedule_idで取得します"
        );

        let recurring_recruitment_rel = scheduled_task_recurring_recruitments::Entity::find()
            .filter(
                scheduled_task_recurring_recruitments::Column::RecruitmentScheduleId
                    .eq(recruitment_schedule_id),
            )
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_schedule_id, "定期募集関連情報の取得に失敗しました");
                e
            })?;

        debug!(
            recruitment_schedule_id,
            found = recurring_recruitment_rel.is_some(),
            "定期募集関連情報を取得しました"
        );
        Ok(recurring_recruitment_rel)
    }

    /// 定期募集タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        scheduled_task_id: i32,
        recruitment_schedule_id: i32,
    ) -> Result<scheduled_task_recurring_recruitments::Model> {
        debug!(
            scheduled_task_id,
            recruitment_schedule_id, "定期募集タスク関連情報を作成します"
        );

        let active_model = scheduled_task_recurring_recruitments::ActiveModel {
            scheduled_task_id: Set(scheduled_task_id),
            recruitment_schedule_id: Set(recruitment_schedule_id),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, scheduled_task_id, recruitment_schedule_id, "定期募集タスク関連情報の作成に失敗しました");
            e
        })?;

        debug!(
            scheduled_task_id,
            recruitment_schedule_id, "定期募集タスク関連情報を作成しました"
        );
        Ok(model)
    }

    /// recruitment_schedule_idで定期募集タスクを削除
    async fn delete_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<u64> {
        debug!(
            recruitment_schedule_id,
            "定期募集タスク関連情報を削除します"
        );

        let result = scheduled_task_recurring_recruitments::Entity::delete_many()
            .filter(
                scheduled_task_recurring_recruitments::Column::RecruitmentScheduleId
                    .eq(recruitment_schedule_id),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_schedule_id, "定期募集タスク関連情報の削除に失敗しました");
                e
            })?;

        debug!(
            recruitment_schedule_id,
            deleted_count = result.rows_affected,
            "定期募集タスク関連情報を削除しました"
        );
        Ok(result.rows_affected)
    }

    /// recruitment_schedule_idに紐づく未実行のscheduled_tasksを削除
    ///
    /// 定期募集スケジュールの削除・無効化時に、まだ実行されていないタスクを削除する
    /// 既に実行済み（is_executed=true）のタスクは削除しない（募集は独立して存在するため）
    async fn delete_pending_tasks_by_recruitment_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_schedule_id: i32,
    ) -> Result<u64> {
        use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;

        debug!(
            recruitment_schedule_id,
            "スケジュールに紐づく未実行scheduled_tasksを削除します"
        );

        // 1. recruitment_schedule_idに紐づくscheduled_task_recurring_recruitmentsを取得
        let recurring_tasks = scheduled_task_recurring_recruitments::Entity::find()
            .filter(
                scheduled_task_recurring_recruitments::Column::RecruitmentScheduleId
                    .eq(recruitment_schedule_id),
            )
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_schedule_id, "定期募集タスク関連情報の取得に失敗しました");
                e
            })?;

        let task_ids: Vec<i32> = recurring_tasks
            .iter()
            .map(|r| r.scheduled_task_id)
            .collect();

        if task_ids.is_empty() {
            debug!(recruitment_schedule_id, "削除対象のタスクがありません");
            return Ok(0);
        }

        // 2. 未実行のscheduled_tasksのみを削除
        let task_repo = SeaOrmScheduledTaskRepository::new();
        let mut deleted_count = 0;

        for task_id in task_ids {
            // タスクの状態を確認
            if let Some(task) = scheduled_tasks::Entity::find_by_id(task_id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, task_id, "タスクの取得に失敗しました");
                    e
                })?
            {
                // 未実行のタスクのみ削除
                if !task.is_executed {
                    task_repo.delete_by_id(txn, task_id).await?;
                    deleted_count += 1;
                    debug!(task_id, "未実行タスクを削除しました");
                } else {
                    debug!(
                        task_id,
                        "実行済みタスクはスキップしました（募集は独立して存在）"
                    );
                }
            }
        }

        debug!(
            recruitment_schedule_id,
            deleted_count, "未実行scheduled_tasksの削除が完了しました"
        );

        Ok(deleted_count)
    }
}

impl SeaOrmScheduledTaskRecurringRecruitmentRepository {
    pub fn new() -> Self {
        Self
    }
}
