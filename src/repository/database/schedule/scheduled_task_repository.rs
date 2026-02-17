use crate::models::entities::worker::{
    scheduled_task_dissolutions,
    scheduled_tasks::{self, TaskExecutionStatus},
};
use crate::repository::schedule::ScheduledTaskRepository as ScheduledTaskRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// スケジュールタスクリポジトリ
#[derive(Debug, Clone, Copy)]
pub struct SeaOrmScheduledTaskRepository;

#[async_trait]
impl ScheduledTaskRepositoryTrait for SeaOrmScheduledTaskRepository {
    /// 指定した日時範囲内の未実行タスクを取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行タスクを取得します"
        );

        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::ExecutionStatus.eq(TaskExecutionStatus::Pending))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "タスクの取得に失敗しました");
                e
            })?;

        debug!(count = tasks.len(), "未実行タスクを取得しました");
        Ok(tasks)
    }

    /// 指定した日時以前の未実行タスクを取得
    async fn find_pending_to(
        &self,
        txn: &DatabaseTransaction,
        to: DateTime<Utc>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        debug!(
            to = %to,
            "指定日時以前の未実行タスクを取得します"
        );

        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::ExecutionStatus.eq(TaskExecutionStatus::Pending))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "タスクの取得に失敗しました");
                e
            })?;

        debug!(count = tasks.len(), "未実行タスクを取得しました");
        Ok(tasks)
    }

    /// IDでタスクを取得（DB再確認用）
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_tasks::Model>> {
        debug!(task_id, "タスクをIDで取得します");

        let task = scheduled_tasks::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "タスクの取得に失敗しました");
                e
            })?;

        Ok(task)
    }

    /// タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        schedule_datetime: DateTime<Utc>,
        task_type: i32,
        guild_id: Option<i64>,
        channel_id: Option<i64>,
    ) -> Result<scheduled_tasks::Model> {
        debug!(
            schedule_datetime = %schedule_datetime,
            task_type = %task_type,
            "タスクを作成します"
        );

        let now = Utc::now();
        let active_model = scheduled_tasks::ActiveModel {
            id: sea_orm::NotSet,
            schedule_datetime: Set(schedule_datetime),
            task_type: Set(task_type),
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            execution_status: Set(TaskExecutionStatus::Pending),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let task = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "タスクの作成に失敗しました");
            e
        })?;

        debug!(task_id = task.id, "タスクを作成しました");
        Ok(task)
    }

    /// タスクを正常終了にマーク
    async fn mark_as_succeeded(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        self.update_execution_status(txn, task_id, TaskExecutionStatus::Succeeded)
            .await
    }

    /// タスクを警告付き正常終了にマーク
    async fn mark_as_succeeded_with_warning(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        self.update_execution_status(txn, task_id, TaskExecutionStatus::SucceededWithWarning)
            .await
    }

    /// タスクを異常終了にマーク
    async fn mark_as_failed(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        self.update_execution_status(txn, task_id, TaskExecutionStatus::Failed)
            .await
    }

    /// タスクの実行状態を更新
    async fn update_execution_status(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        status: TaskExecutionStatus,
    ) -> Result<scheduled_tasks::Model> {
        debug!(task_id, status = ?status, "タスクの実行状態を更新します");

        let task = scheduled_tasks::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "タスクの取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(task_id, "タスクが見つかりません");
                crate::types::AppError::Business {
                    message: format!("タスクが見つかりません: {task_id}"),
                }
            })?;

        let mut active_model: scheduled_tasks::ActiveModel = task.into();
        active_model.execution_status = Set(status);
        active_model.updated_at = Set(Utc::now());

        let updated_task = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, task_id, "タスクの更新に失敗しました");
            e
        })?;

        debug!(task_id, status = ?updated_task.execution_status, "タスクの実行状態を更新しました");
        Ok(updated_task)
    }

    /// IDでタスクを削除
    async fn delete_by_id(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<u64> {
        debug!(task_id, "タスクを削除します");

        let delete_result = scheduled_tasks::Entity::delete_by_id(task_id)
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "タスクの削除に失敗しました");
                e
            })?;

        debug!(
            task_id,
            deleted_count = delete_result.rows_affected,
            "タスクを削除しました"
        );

        Ok(delete_result.rows_affected)
    }

    /// recruit_idに紐づく解散タスクを削除
    async fn delete_dissolutions_by_recruit_id(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<u64> {
        debug!(recruit_id, "recruit_idに紐づく解散タスクを削除します");

        // scheduled_task_dissolutions から task_id を取得
        let dissolution_tasks = scheduled_task_dissolutions::Entity::find()
            .filter(scheduled_task_dissolutions::Column::RecruitId.eq(recruit_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruit_id, "解散タスクの取得に失敗しました");
                e
            })?;

        let task_ids: Vec<i32> = dissolution_tasks.iter().map(|d| d.task_id).collect();

        if task_ids.is_empty() {
            debug!(recruit_id, "削除対象の解散タスクはありません");
            return Ok(0);
        }

        // scheduled_tasks を削除（CASCADE で dissolution も削除される）
        let delete_result = scheduled_tasks::Entity::delete_many()
            .filter(scheduled_tasks::Column::Id.is_in(task_ids.clone()))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruit_id, "解散タスクの削除に失敗しました");
                e
            })?;

        debug!(
            recruit_id,
            deleted_count = delete_result.rows_affected,
            "解散タスクを削除しました"
        );

        Ok(delete_result.rows_affected)
    }

    /// 指定したtask_typeのタスクを全て削除
    async fn delete_all_by_task_type(
        &self,
        txn: &DatabaseTransaction,
        task_type: i32,
    ) -> Result<u64> {
        debug!(task_type, "指定したtask_typeのタスクを全て削除します");

        let delete_result = scheduled_tasks::Entity::delete_many()
            .filter(scheduled_tasks::Column::TaskType.eq(task_type))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_type, "タスクの削除に失敗しました");
                e
            })?;

        debug!(
            task_type,
            deleted_count = delete_result.rows_affected,
            "タスクを削除しました"
        );

        Ok(delete_result.rows_affected)
    }

    /// 指定したguildの指定task_typeタスクを全て削除
    async fn delete_all_by_task_type_and_guild(
        &self,
        txn: &DatabaseTransaction,
        task_type: i32,
        guild_id: i64,
    ) -> Result<u64> {
        debug!(
            task_type,
            guild_id, "指定したguildのtask_typeタスクを全て削除します"
        );

        let delete_result = scheduled_tasks::Entity::delete_many()
            .filter(scheduled_tasks::Column::TaskType.eq(task_type))
            .filter(scheduled_tasks::Column::GuildId.eq(Some(guild_id)))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_type, guild_id, "タスクの削除に失敗しました");
                e
            })?;

        debug!(
            task_type,
            guild_id,
            deleted_count = delete_result.rows_affected,
            "タスクを削除しました"
        );

        Ok(delete_result.rows_affected)
    }

    async fn find_many_by_ids_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        Self::find_many_by_ids_internal(txn, ids).await
    }

    async fn find_many_by_ids_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>> {
        Self::find_many_by_ids_internal(db, ids).await
    }

    async fn delete_before_date_with_txn(
        &self,
        txn: &DatabaseTransaction,
        before: DateTime<Utc>,
    ) -> Result<u64> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        debug!(%before, "指定日時より前のタスクを削除します");

        let delete_result = scheduled_tasks::Entity::delete_many()
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(before))
            .filter(scheduled_tasks::Column::ExecutionStatus.ne(TaskExecutionStatus::Pending))
            .filter(
                scheduled_tasks::Column::TaskType
                    .ne(scheduled_tasks::ScheduledTaskType::DataCleanup.as_i32()),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, %before, "タスクの削除に失敗しました");
                e
            })?;

        debug!(
            %before,
            deleted_count = delete_result.rows_affected,
            "タスクを削除しました"
        );

        Ok(delete_result.rows_affected)
    }
}

impl SeaOrmScheduledTaskRepository {
    /// 複数IDでタスクを取得する内部実装
    async fn find_many_by_ids_internal<'c, C>(
        db: &'c C,
        ids: Vec<i32>,
    ) -> Result<Vec<scheduled_tasks::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        if ids.is_empty() {
            return Ok(vec![]);
        }

        debug!(?ids, "複数IDでタスクを取得します");

        scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::Id.is_in(ids))
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, "タスクの取得に失敗しました");
                e.into()
            })
    }

    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmScheduledTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}
