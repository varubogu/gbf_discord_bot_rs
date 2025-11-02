use crate::models::entities::{event_schedules, event_schedules::Entity as EventScheduleEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchedule {
    pub id: Uuid,
    pub event_type: String,
    pub event_count: i64,
    pub profile: String,
    pub weak_attribute: i32,
    pub start_at: chrono::DateTime<chrono::Utc>,
    pub end_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<event_schedules::Model> for EventSchedule {
    fn from(model: event_schedules::Model) -> Self {
        Self {
            id: model.id,
            event_type: model.event_type,
            event_count: model.event_count,
            profile: model.profile,
            weak_attribute: model.weak_attribute,
            start_at: model.start_at,
            end_at: model.end_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_event_schedules(&self) -> Result<Vec<EventSchedule>, DbErr> {
        let models = EventScheduleEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_event_schedule_by_id(&self, id: Uuid) -> Result<Option<EventSchedule>, DbErr> {
        let event_schedule = EventScheduleEntity::find()
            .filter(event_schedules::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(event_schedule.map(|es| es.into()))
    }

    pub async fn get_event_schedules_by_profile(
        &self,
        profile: &str,
    ) -> Result<Vec<EventSchedule>, DbErr> {
        let models = EventScheduleEntity::find()
            .filter(event_schedules::Column::Profile.eq(profile))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_event_schedule_by_type_and_count(
        &self,
        event_type: &str,
        event_count: i64,
    ) -> Result<Option<EventSchedule>, DbErr> {
        let event_schedule = EventScheduleEntity::find()
            .filter(event_schedules::Column::EventType.eq(event_type))
            .filter(event_schedules::Column::EventCount.eq(event_count))
            .one(&self.conn)
            .await?;

        Ok(event_schedule.map(|es| es.into()))
    }
}
