use crate::models::entities::{MessageText as MessageTextEntity, message_texts};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTexts {
    pub id: i32,
    pub guild_id: i64,
    pub message_id: String,
    pub message_jp: String,
    pub message_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<message_texts::Model> for MessageTexts {
    fn from(model: message_texts::Model) -> Self {
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
    ) -> Result<Option<MessageTexts>, DbErr> {
        let model = MessageTextEntity::find()
            .filter(message_texts::Column::GuildId.eq(guild_id))
            .filter(message_texts::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await?;

        Ok(model.map(|m| m.into()))
    }
}
