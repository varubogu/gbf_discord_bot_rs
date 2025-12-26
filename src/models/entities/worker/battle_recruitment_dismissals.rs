use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "battle_recruitment_dismissals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub recruitment_id: i32,
    pub input_value: String,
    pub input_type: i32,
    pub dismissal_datetime: Option<DateTimeUtc>,
    pub relative_days: Option<i32>,
    pub relative_hours: Option<i32>,
    pub relative_minutes: Option<i32>,
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
            recruitment_id: sea_orm::NotSet,
            input_value: sea_orm::NotSet,
            input_type: sea_orm::NotSet,
            dismissal_datetime: sea_orm::NotSet,
            relative_days: sea_orm::NotSet,
            relative_hours: sea_orm::NotSet,
            relative_minutes: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}

/// 解散時刻の入力タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DismissalInputType {
    Absolute = 1, // 絶対日時
    Relative = 2, // 相対時刻
}

impl DismissalInputType {
    #[allow(dead_code)]
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    #[allow(dead_code)]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Absolute),
            2 => Some(Self::Relative),
            _ => None,
        }
    }
}
