use crate::models::entities::worker::{scheduled_task_dismissals, scheduled_tasks};
use crate::types::{AppError, Result};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 解散タスクと解散設定の関連情報
#[derive(Debug, Clone)]
pub struct DismissalWithTask {
    pub task: scheduled_tasks::Model,
    pub dismissal_rel: scheduled_task_dismissals::Model,
}

/// 解散タスクリポジトリ
#[derive(Default)]
pub struct SeaOrmScheduledTaskDismissalRepository;

impl SeaOrmScheduledTaskDismissalRepository {
    pub fn new() -> Self {
        Self
    }

    /// 指定範囲内の未実行解散タスクをJOIN済みで取得
    pub async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DismissalWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行解散タスクを取得します"
        );

        // scheduled_tasks と scheduled_task_dismissals を手動でJOIN
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
            .filter(scheduled_tasks::Column::TaskType.eq(2)) // Dismissal
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "解散タスクの取得に失敗しました");
                AppError::Database(e)
            })?;

        let mut results = Vec::new();
        for task in tasks {
            // 各タスクに対して dismissal_rel 情報を取得
            if let Some(dismissal_rel) = scheduled_task_dismissals::Entity::find_by_id(task.id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, task_id = task.id, "解散関連情報の取得に失敗しました");
                    AppError::Database(e)
                })?
            {
                results.push(DismissalWithTask {
                    task,
                    dismissal_rel,
                });
            }
        }

        debug!(count = results.len(), "未実行解散タスクを取得しました");
        Ok(results)
    }

    /// task_idで解散関連情報を取得
    pub async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_dismissals::Model>> {
        debug!(task_id, "解散関連情報をtask_idで取得します");

        let dismissal_rel = scheduled_task_dismissals::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "解散関連情報の取得に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            task_id,
            found = dismissal_rel.is_some(),
            "解散関連情報を取得しました"
        );
        Ok(dismissal_rel)
    }

    /// recruitment_dismissal_idで解散関連情報を取得
    pub async fn find_by_recruitment_dismissal_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_dismissal_id: i32,
    ) -> Result<Option<scheduled_task_dismissals::Model>> {
        debug!(
            recruitment_dismissal_id,
            "解散関連情報をrecruitment_dismissal_idで取得します"
        );

        let dismissal_rel = scheduled_task_dismissals::Entity::find()
            .filter(
                scheduled_task_dismissals::Column::RecruitmentDismissalId
                    .eq(recruitment_dismissal_id),
            )
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_dismissal_id, "解散関連情報の取得に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            recruitment_dismissal_id,
            found = dismissal_rel.is_some(),
            "解散関連情報を取得しました"
        );
        Ok(dismissal_rel)
    }

    /// 解散タスクを作成
    pub async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruitment_dismissal_id: i32,
    ) -> Result<scheduled_task_dismissals::Model> {
        debug!(
            task_id,
            recruitment_dismissal_id, "解散タスク関連情報を作成します"
        );

        let now = chrono::Utc::now();
        let active_model = scheduled_task_dismissals::ActiveModel {
            task_id: Set(task_id),
            recruitment_dismissal_id: Set(recruitment_dismissal_id),
            created_at: Set(now),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, task_id, recruitment_dismissal_id, "解散タスク関連情報の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            task_id,
            recruitment_dismissal_id, "解散タスク関連情報を作成しました"
        );
        Ok(model)
    }

    /// recruitment_dismissal_idで解散タスクを削除
    pub async fn delete_by_recruitment_dismissal_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_dismissal_id: i32,
    ) -> Result<u64> {
        debug!(recruitment_dismissal_id, "解散タスク関連情報を削除します");

        let result = scheduled_task_dismissals::Entity::delete_many()
            .filter(
                scheduled_task_dismissals::Column::RecruitmentDismissalId
                    .eq(recruitment_dismissal_id),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_dismissal_id, "解散タスク関連情報の削除に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            recruitment_dismissal_id,
            deleted_count = result.rows_affected,
            "解散タスク関連情報を削除しました"
        );
        Ok(result.rows_affected)
    }
}
