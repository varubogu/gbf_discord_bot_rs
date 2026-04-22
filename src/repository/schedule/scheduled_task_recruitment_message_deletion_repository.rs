use crate::models::entities::worker::{
    scheduled_task_recruitment_message_deletions, scheduled_tasks,
};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 募集投稿削除タスクと募集の関連情報
#[derive(Debug, Clone)]
pub struct RecruitmentMessageDeletionWithTask {
    pub task: scheduled_tasks::Model,
    pub deletion: scheduled_task_recruitment_message_deletions::Model,
}

/// 募集投稿削除タスクリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduledTaskRecruitmentMessageDeletionRepository: Send + Sync {
    /// 指定範囲内の未実行募集投稿削除タスクを取得
    async fn find_pending_in_range(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RecruitmentMessageDeletionWithTask>>;

    /// task_idで募集投稿削除情報を取得
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_recruitment_message_deletions::Model>>;

    /// recruitment_idで募集投稿削除情報を取得
    async fn find_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<scheduled_task_recruitment_message_deletions::Model>>;

    /// 募集投稿削除タスクを作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        recruitment_id: i32,
    ) -> Result<scheduled_task_recruitment_message_deletions::Model>;
}
