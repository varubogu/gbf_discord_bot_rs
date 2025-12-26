use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "guild_master",
    table_name = "battle_recruitment_schedule_dismissals"
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub schedule_id: i32,
    pub input_value: String,
    pub input_type: i32,
    pub dismissal_time: Option<TimeTime>,
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
            schedule_id: sea_orm::NotSet,
            input_value: sea_orm::NotSet,
            input_type: sea_orm::NotSet,
            dismissal_time: sea_orm::NotSet,
            relative_days: sea_orm::NotSet,
            relative_hours: sea_orm::NotSet,
            relative_minutes: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
