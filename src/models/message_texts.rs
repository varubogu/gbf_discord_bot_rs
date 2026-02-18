use crate::models::entities::{MessageText as MessageTextEntity, master::message_texts};
use crate::infrastructure::database::repositories::db_compat::Database;
use sea_orm::{DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTexts {
    pub id: String,
    pub message_jp: String,
    pub message_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<message_texts::Model> for MessageTexts {
    fn from(model: message_texts::Model) -> Self {
        Self {
            id: model.id,
            message_jp: model.message_jp,
            message_en: model.message_en,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_message_text(&self, message_id: &str) -> Result<Option<MessageTexts>, DbErr> {
        let model = MessageTextEntity::find_by_id(message_id.to_string())
            .one(&self.conn)
            .await?;

        Ok(model.map(|m| m.into()))
    }
}
