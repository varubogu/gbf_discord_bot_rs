use crate::models::entities::{notifications, notifications::Entity as NotificationEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub schedule_datetime: chrono::DateTime<chrono::Utc>,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_text_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<notifications::Model> for Notification {
    fn from(model: notifications::Model) -> Self {
        Self {
            id: model.id,
            schedule_datetime: model.schedule_datetime,
            guild_id: model.guild_id,
            channel_id: model.channel_id,
            message_text_id: model.message_text_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_notifications(&self) -> Result<Vec<Notification>, DbErr> {
        let models = NotificationEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_by_id(&self, id: i32) -> Result<Option<Notification>, DbErr> {
        let notification = NotificationEntity::find()
            .filter(notifications::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(notification.map(|n| n.into()))
    }

    pub async fn get_notifications_by_guild(
        &self,
        guild_id: i64,
    ) -> Result<Vec<Notification>, DbErr> {
        let models = NotificationEntity::find()
            .filter(notifications::Column::GuildId.eq(guild_id))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
