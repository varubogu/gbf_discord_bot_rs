use crate::models::entities::worker::{
    scheduled_task_recruitment_message_deletions,
    scheduled_tasks::{self, TaskExecutionStatus},
};
use crate::repository::schedule::{
    RecruitmentMessageDeletionWithTask, ScheduledTaskRecruitmentMessageDeletionRepository,
};
use crate::types::{AppError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 募集投稿削除タスクリポジトリ
#[derive(Default, Debug, Clone, Copy)]
pub struct SeaOrmScheduledTaskRecruitmentMessageDeletionRepository;

#[async_trait]
impl ScheduledTaskRecruitmentMessageDeletionRepository
    for SeaOrmScheduledTaskRecruitmentMessageDeletionRepository
{
    /// 指定範囲内の未実行募集投稿削除タスクをJOIN済みで取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RecruitmentMessageDeletionWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行募集投稿削除タスクを取得します"
        );

        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::ExecutionStatus.eq(TaskExecutionStatus::Pending))
            .filter(
                scheduled_tasks::Column::TaskType
                    .eq(scheduled_tasks::ScheduledTaskType::RecruitmentMessageDeletion.as_i32()),
            )
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "募集投稿削除タスクの取得に失敗しました");
                AppError::Database(e)
            })?;

        let mut results = Vec::new();
        for task in tasks {
            if let Some(deletion) =
                scheduled_task_recruitment_message_deletions::Entity::find_by_id(task.id)
                    .one(txn)
                    .await
                    .map_err(|e| {
                        error!(error = %e, task_id = task.id, "募集投稿削除関連情報の取得に失敗しました");
                        AppError::Database(e)
                    })?
            {
                results.push(RecruitmentMessageDeletionWithTask { task, deletion });
            }
        }

        debug!(
            count = results.len(),
            "未実行募集投稿削除タスクを取得しました"
        );
        Ok(results)
    }

    /// task_idで募集投稿削除情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_recruitment_message_deletions::Model>> {
        debug!(task_id, "募集投稿削除関連情報をtask_idで取得します");

        let deletion = scheduled_task_recruitment_message_deletions::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "募集投稿削除関連情報の取得に失敗しました");
                AppError::Database(e)
            })?;

        Ok(deletion)
    }

    /// recruitment_idで募集投稿削除情報を取得
    async fn find_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<scheduled_task_recruitment_message_deletions::Model>> {
        debug!(
            recruitment_id,
            "募集投稿削除関連情報をrecruitment_idで取得します"
        );

        let deletions = scheduled_task_recruitment_message_deletions::Entity::find()
            .filter(
                scheduled_task_recruitment_message_deletions::Column::RecruitmentId
                    .eq(recruitment_id),
            )
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_id, "募集投稿削除関連情報の取得に失敗しました");
                AppError::Database(e)
            })?;

        Ok(deletions)
    }

    /// 募集投稿削除タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruitment_id: i32,
    ) -> Result<scheduled_task_recruitment_message_deletions::Model> {
        debug!(task_id, recruitment_id, "募集投稿削除関連情報を作成します");

        let active_model = scheduled_task_recruitment_message_deletions::ActiveModel {
            task_id: Set(task_id),
            recruitment_id: Set(recruitment_id),
            created_at: Set(Utc::now()),
        };

        let deletion = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, task_id, recruitment_id, "募集投稿削除関連情報の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            task_id,
            recruitment_id, "募集投稿削除関連情報を作成しました"
        );
        Ok(deletion)
    }
}

impl SeaOrmScheduledTaskRecruitmentMessageDeletionRepository {
    pub fn new() -> Self {
        Self
    }
}
