use crate::models::entities::{
    notification_rel_event_schedules,
    notification_rel_event_schedules::Entity as NotificationRelEventScheduleEntity,
};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRelEventSchedule {
    pub event_schedule_id: Uuid,
    pub event_schedule_detail_id: Option<Uuid>,
    pub notification_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<notification_rel_event_schedules::Model> for NotificationRelEventSchedule {
    fn from(model: notification_rel_event_schedules::Model) -> Self {
        Self {
            event_schedule_id: model.event_schedule_id,
            event_schedule_detail_id: model.event_schedule_detail_id,
            notification_id: model.notification_id,
            created_at: model.created_at,
        }
    }
}

impl Database {
    pub async fn get_notification_rel_event_schedules(
        &self,
    ) -> Result<Vec<NotificationRelEventSchedule>, DbErr> {
        let models = NotificationRelEventScheduleEntity::find()
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_rel_event_schedule_by_ids(
        &self,
        event_schedule_id: Uuid,
        notification_id: i32,
    ) -> Result<Option<NotificationRelEventSchedule>, DbErr> {
        let relation = NotificationRelEventScheduleEntity::find()
            .filter(notification_rel_event_schedules::Column::EventScheduleId.eq(event_schedule_id))
            .filter(notification_rel_event_schedules::Column::NotificationId.eq(notification_id))
            .one(&self.conn)
            .await?;

        Ok(relation.map(|r| r.into()))
    }

    pub async fn get_notification_rel_event_schedules_by_event_schedule(
        &self,
        event_schedule_id: Uuid,
    ) -> Result<Vec<NotificationRelEventSchedule>, DbErr> {
        let models = NotificationRelEventScheduleEntity::find()
            .filter(notification_rel_event_schedules::Column::EventScheduleId.eq(event_schedule_id))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_rel_event_schedules_by_notification(
        &self,
        notification_id: i32,
    ) -> Result<Vec<NotificationRelEventSchedule>, DbErr> {
        let models = NotificationRelEventScheduleEntity::find()
            .filter(notification_rel_event_schedules::Column::NotificationId.eq(notification_id))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_rel_event_schedules_by_detail(
        &self,
        event_schedule_detail_id: Uuid,
    ) -> Result<Vec<NotificationRelEventSchedule>, DbErr> {
        let models = NotificationRelEventScheduleEntity::find()
            .filter(
                notification_rel_event_schedules::Column::EventScheduleDetailId
                    .eq(event_schedule_detail_id),
            )
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
