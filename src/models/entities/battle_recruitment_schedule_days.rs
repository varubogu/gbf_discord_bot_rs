use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "worker",
    table_name = "battle_recruitment_schedule_days"
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub schedule_id: i32,
    pub day_of_week: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::battle_recruitment_schedules::Entity",
        from = "Column::ScheduleId",
        to = "super::battle_recruitment_schedules::Column::Id"
    )]
    BattleRecruitmentSchedule,
}

impl Related<super::battle_recruitment_schedules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitmentSchedule.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            schedule_id: sea_orm::NotSet,
            day_of_week: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
