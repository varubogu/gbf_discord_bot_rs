use crate::models::entities::master::{quest_aliases, quests};
use crate::models::entities::{Quest as QuestEntity, QuestAlias as QuestAliasEntity};
use crate::models::quests::Quest;
use crate::infrastructure::database::session::DatabaseSession as Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestAlias {
    pub quest_id: i32,
    pub sequence_no: i32,
    pub alias: String,
    pub alias_kana_small: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<quest_aliases::Model> for QuestAlias {
    fn from(model: quest_aliases::Model) -> Self {
        Self {
            quest_id: model.quest_id,
            sequence_no: model.sequence_no,
            alias: model.alias,
            alias_kana_small: model.alias_kana_small,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_quest_aliases(&self) -> Result<Vec<QuestAlias>, DbErr> {
        let models = QuestAliasEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_quest_by_alias(&self, alias: &str) -> Result<Option<Quest>, DbErr> {
        let quest_alias = QuestAliasEntity::find()
            .filter(quest_aliases::Column::Alias.eq(alias))
            .one(&self.conn)
            .await?;

        if let Some(qa) = quest_alias {
            let quest = QuestEntity::find()
                .filter(quests::Column::Id.eq(qa.quest_id))
                .one(&self.conn)
                .await?;

            return Ok(quest.map(|q| q.into()));
        }

        Ok(None)
    }
}
