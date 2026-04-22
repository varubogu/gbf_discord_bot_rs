use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "worker",
    table_name = "scheduled_task_recruitment_message_deletions"
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    pub recruitment_id: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scheduled_tasks::Entity",
        from = "Column::TaskId",
        to = "super::scheduled_tasks::Column::Id"
    )]
    ScheduledTask,
    #[sea_orm(
        belongs_to = "super::battle_recruitments::Entity",
        from = "Column::RecruitmentId",
        to = "super::battle_recruitments::Column::Id"
    )]
    BattleRecruitment,
}

impl Related<super::scheduled_tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ScheduledTask.def()
    }
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
