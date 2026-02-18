use crate::models::entities::master::event_schedule_details::{
    self, Entity as EventScheduleDetailEntity,
};
use crate::infrastructure::database::repositories::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventScheduleDetail {
    pub id: Uuid,
    pub profile: String,
    pub start_day_relative: String,
    pub time: String,
    pub schedule_name: String,
    pub message_text_id: String,
    pub notification_channel_type: i32,
    pub reactions: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<event_schedule_details::Model> for EventScheduleDetail {
    fn from(model: event_schedule_details::Model) -> Self {
        Self {
            id: model.id,
            profile: model.profile,
            start_day_relative: model.start_day_relative,
            time: model.time,
            schedule_name: model.schedule_name,
            message_text_id: model.message_text_id,
            notification_channel_type: model.notification_channel_type,
            reactions: model.reactions,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_event_schedule_details(&self) -> Result<Vec<EventScheduleDetail>, DbErr> {
        let models = EventScheduleDetailEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_event_schedule_detail_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<EventScheduleDetail>, DbErr> {
        let event_schedule_detail = EventScheduleDetailEntity::find()
            .filter(event_schedule_details::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(event_schedule_detail.map(|esd| esd.into()))
    }

    pub async fn get_event_schedule_details_by_profile(
        &self,
        profile: &str,
    ) -> Result<Vec<EventScheduleDetail>, DbErr> {
        let models = EventScheduleDetailEntity::find()
            .filter(event_schedule_details::Column::Profile.eq(profile))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
