use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "worker",
    table_name = "notification_rel_event_schedules"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub event_schedule_id: Uuid,
    pub event_schedule_detail_id: Option<Uuid>,
    #[sea_orm(primary_key, auto_increment = false)]
    pub notification_id: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::notifications::Entity",
        from = "Column::NotificationId",
        to = "super::notifications::Column::Id"
    )]
    Notification,
    #[sea_orm(
        belongs_to = "super::super::master::event_schedules::Entity",
        from = "Column::EventScheduleId",
        to = "super::super::master::event_schedules::Column::Id"
    )]
    EventSchedule,
    #[sea_orm(
        belongs_to = "super::super::master::event_schedule_details::Entity",
        from = "Column::EventScheduleDetailId",
        to = "super::super::master::event_schedule_details::Column::Id"
    )]
    EventScheduleDetail,
}

impl Related<super::notifications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notification.def()
    }
}

impl Related<super::super::master::event_schedules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventSchedule.def()
    }
}

impl Related<super::super::master::event_schedule_details::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventScheduleDetail.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            event_schedule_id: sea_orm::NotSet,
            event_schedule_detail_id: sea_orm::NotSet,
            notification_id: sea_orm::NotSet,
            created_at: sea_orm::Set(chrono::Utc::now()),
        }
    }
}
