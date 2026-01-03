use crate::models::entities::worker::{scheduled_task_dissolutions, scheduled_tasks};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 解散タスクと募集の関連情報
#[derive(Debug, Clone)]
pub struct DissolutionWithTask {
    pub task: scheduled_tasks::Model,
    pub dissolution: scheduled_task_dissolutions::Model,
}

/// 解散タスクリポジトリ
pub struct SeaOrmScheduledTaskDissolutionRepository;

impl SeaOrmScheduledTaskDissolutionRepository {
    pub fn new() -> Self {
        Self
    }

    /// 指定範囲内の未実行解散タスクをJOIN済みで取得
    pub async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DissolutionWithTask>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未実行解散タスクを取得します"
        );

        // scheduled_tasks と scheduled_task_dissolutions を手動でJOIN
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::IsExecuted.eq(false))
            .filter(scheduled_tasks::Column::TaskType.eq(2)) // Dissolution
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "解散タスクの取得に失敗しました");
                e
            })?;

        let mut results = Vec::new();
        for task in tasks {
            // 各タスクに対して dissolution 情報を取得
            if let Some(dissolution) = scheduled_task_dissolutions::Entity::find_by_id(task.id)
                .one(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, task_id = task.id, "解散情報の取得に失敗しました");
                    e
                })?
            {
                results.push(DissolutionWithTask { task, dissolution });
            }
        }

        debug!(count = results.len(), "未実行解散タスクを取得しました");
        Ok(results)
    }

    /// task_idで解散情報を取得
    pub async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_dissolutions::Model>> {
        debug!(task_id, "解散情報をtask_idで取得します");

        let dissolution = scheduled_task_dissolutions::Entity::find_by_id(task_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id, "解散情報の取得に失敗しました");
                e
            })?;

        Ok(dissolution)
    }

    /// 解散タスクを作成
    pub async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruit_id: i32,
    ) -> Result<scheduled_task_dissolutions::Model> {
        debug!(task_id, recruit_id, "解散タスクを作成します");

        let active_model = scheduled_task_dissolutions::ActiveModel {
            task_id: Set(task_id),
            recruit_id: Set(recruit_id),
        };

        let dissolution = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "解散タスクの作成に失敗しました");
            e
        })?;

        debug!(task_id, recruit_id, "解散タスクを作成しました");
        Ok(dissolution)
    }

    /// recruit_idで解散タスクを取得
    pub async fn find_by_recruit_id(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<Vec<scheduled_task_dissolutions::Model>> {
        debug!(recruit_id, "recruit_idで解散タスクを取得します");

        let dissolutions = scheduled_task_dissolutions::Entity::find()
            .filter(scheduled_task_dissolutions::Column::RecruitId.eq(recruit_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruit_id, "解散タスクの取得に失敗しました");
                e
            })?;

        debug!(
            recruit_id,
            count = dissolutions.len(),
            "解散タスクを取得しました"
        );
        Ok(dissolutions)
    }
}

impl Default for SeaOrmScheduledTaskDissolutionRepository {
    fn default() -> Self {
        Self::new()
    }
}
