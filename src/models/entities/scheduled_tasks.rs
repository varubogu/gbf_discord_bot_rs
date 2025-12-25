use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "scheduled_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub schedule_datetime: DateTimeUtc,
    pub task_type: i32,
    pub guild_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub is_executed: bool,
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
            is_executed: sea_orm::Set(false),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}

// タスク種別のenum定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScheduledTaskType {
    Notification = 1,         // 通知
    Dissolution = 2,          // 解散
    DataCleanup = 3,          // データクリーンアップ
    RecurringRecruitment = 4, // 定期募集
}

impl ScheduledTaskType {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    #[warn(dead_code)]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Notification),
            2 => Some(Self::Dissolution),
            3 => Some(Self::DataCleanup),
            4 => Some(Self::RecurringRecruitment),
            _ => None,
        }
    }

    #[warn(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Notification => "通知",
            Self::Dissolution => "解散",
            Self::DataCleanup => "データ整理",
            Self::RecurringRecruitment => "定期募集",
        }
    }
}
