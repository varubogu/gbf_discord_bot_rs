use crate::models::entities::worker::{scheduled_task_dissolutions, scheduled_tasks};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// スケジュールタスクリポジトリ
pub struct SeaOrmScheduledTaskRepository;

impl SeaOrmScheduledTaskRepository {
    pub fn new() -> Self {
        Self
    }

    /// 指定した日時範囲内の未実行タスクを取得
    pub async fn find_pending_in_range(
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
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
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
    pub async fn find_pending_to(
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
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
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
    pub async fn find_by_id(
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
    pub async fn create(
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
            is_executed: Set(false),
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

    /// タスクを実行済みにマーク
    pub async fn mark_as_executed(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<scheduled_tasks::Model> {
        debug!(task_id, "タスクを実行済みにマークします");

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
        active_model.is_executed = Set(true);
        active_model.updated_at = Set(Utc::now());

        let updated_task = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, task_id, "タスクの更新に失敗しました");
            e
        })?;

        debug!(task_id, "タスクを実行済みにマークしました");
        Ok(updated_task)
    }

    /// IDでタスクを削除
    pub async fn delete_by_id(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<u64> {
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
    pub async fn delete_dissolutions_by_recruit_id(
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
}

impl Default for SeaOrmScheduledTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}
