use crate::models::entities::Quest as QuestEntity;
use crate::models::entities::quests;
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: i32,
    pub name: String,
    pub default_battle_style: i32,
    pub recruit_count: i32,
    pub available_battle_styles: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<quests::Model> for Quest {
    fn from(model: quests::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            default_battle_style: model.default_battle_style,
            recruit_count: model.recruit_count,
            available_battle_styles: model.available_battle_styles,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_quests(&self) -> Result<Vec<Quest>, DbErr> {
        let models = QuestEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_quest_by_id(&self, id: i32) -> Result<Option<Quest>, DbErr> {
        let quest = QuestEntity::find()
            .filter(quests::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(quest.map(|q| q.into()))
    }
}
