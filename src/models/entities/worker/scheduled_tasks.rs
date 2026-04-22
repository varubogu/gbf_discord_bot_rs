use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    // search_path に依存せず、常に worker スキーマの ENUM を参照する。
    // SeaQuery(Postgres)のCASTは型名全体を "..." で囲むため、
    // `worker.task_execution_status` を渡すと `"worker.task_execution_status"` となってしまう。
    // そのため `"worker"."task_execution_status"` になるように識別子内にクォートを埋め込む。
    enum_name = "worker\".\"task_execution_status"
)]
pub enum TaskExecutionStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "succeeded")]
    Succeeded,
    #[sea_orm(string_value = "succeeded_with_warning")]
    SucceededWithWarning,
    #[sea_orm(string_value = "failed")]
    Failed,
}

impl TaskExecutionStatus {
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "scheduled_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub schedule_datetime: DateTimeUtc,
    pub task_type: i32,
    pub guild_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub execution_status: TaskExecutionStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            schedule_datetime: sea_orm::NotSet,
            task_type: sea_orm::NotSet,
            guild_id: sea_orm::NotSet,
            channel_id: sea_orm::NotSet,
            execution_status: sea_orm::Set(TaskExecutionStatus::Pending),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}

// タスク種別のenum定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScheduledTaskType {
    Notification = 1,               // 通知
    Dissolution = 2,                // 解散
    DataCleanup = 3,                // データクリーンアップ
    RecurringRecruitment = 4,       // 定期募集
    Dismissal = 5,                  // 人数不足解散
    AutoRecruitmentRotation = 6,    // 自動募集日付ローテーション
    AutoMatching = 7,               // 自動マッチング
    RecruitmentMessageDeletion = 8, // 募集投稿削除
}

impl ScheduledTaskType {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    #[allow(dead_code)]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Notification),
            2 => Some(Self::Dissolution),
            3 => Some(Self::DataCleanup),
            4 => Some(Self::RecurringRecruitment),
            5 => Some(Self::Dismissal),
            6 => Some(Self::AutoRecruitmentRotation),
            7 => Some(Self::AutoMatching),
            8 => Some(Self::RecruitmentMessageDeletion),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Notification => "通知",
            Self::Dissolution => "解散",
            Self::DataCleanup => "データ整理",
            Self::RecurringRecruitment => "定期募集",
            Self::Dismissal => "人数不足解散",
            Self::AutoRecruitmentRotation => "自動募集日付ローテーション",
            Self::AutoMatching => "自動マッチング",
            Self::RecruitmentMessageDeletion => "募集投稿削除",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskExecutionStatus;

    #[test]
    fn test_is_pending() {
        assert!(TaskExecutionStatus::Pending.is_pending());
        assert!(!TaskExecutionStatus::Succeeded.is_pending());
        assert!(!TaskExecutionStatus::SucceededWithWarning.is_pending());
        assert!(!TaskExecutionStatus::Failed.is_pending());
    }
}
