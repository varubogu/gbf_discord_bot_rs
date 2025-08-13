use serde::{Deserialize, Serialize};
use crate::models::entities::{message_text, MessageText as MessageTextEntity};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DbErr};
use crate::models::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageText {
    pub id: i32,
    pub guild_id: i64,
    pub message_id: String,
    pub message_jp: String,
    pub message_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<message_text::Model> for MessageText {
    fn from(model: message_text::Model) -> Self {
        Self {
            id: model.id,
            guild_id: model.guild_id,
            message_id: model.message_id,
            message_jp: model.message_jp,
            message_en: model.message_en,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_message_text(
        &self,
        guild_id: i64,
        message_id: &str,
    ) -> Result<Option<MessageText>, DbErr> {
        let model = MessageTextEntity::find()
            .filter(message_text::Column::GuildId.eq(guild_id))
            .filter(message_text::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await?;
            
        Ok(model.map(|m| m.into()))
    }
}