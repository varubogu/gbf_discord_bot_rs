use crate::models::entities::Quest as QuestEntity;
use crate::models::entities::quests;
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: i32,
    pub target_id: i32,
    pub quest_name: String,
    pub default_battle_type: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<quests::Model> for Quest {
    fn from(model: quests::Model) -> Self {
        Self {
            id: model.id,
            target_id: model.target_id,
            quest_name: model.quest_name,
            default_battle_type: model.default_battle_type,
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

    pub async fn get_quest_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, DbErr> {
        let quest = QuestEntity::find()
            .filter(quests::Column::TargetId.eq(target_id))
            .one(&self.conn)
            .await?;

        Ok(quest.map(|q| q.into()))
    }
}
