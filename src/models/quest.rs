use crate::models::{Database, Quest, QuestAlias};
use crate::models::entities::{quest, quest_alias};
use crate::models::entities::{Quest as QuestEntity, QuestAlias as QuestAliasEntity};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, RelationTrait, DbErr};

impl From<quest::Model> for Quest {
    fn from(model: quest::Model) -> Self {
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

impl From<quest_alias::Model> for QuestAlias {
    fn from(model: quest_alias::Model) -> Self {
        Self {
            id: model.id,
            target_id: model.target_id,
            alias: model.alias,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_quests(&self) -> Result<Vec<Quest>, DbErr> {
        let models = QuestEntity::find()
            .all(&self.conn)
            .await?;
            
        Ok(models.into_iter().map(|model| model.into()).collect())
    }
    
    pub async fn get_quest_aliases(&self) -> Result<Vec<QuestAlias>, DbErr> {
        let models = QuestAliasEntity::find()
            .all(&self.conn)
            .await?;
            
        Ok(models.into_iter().map(|model| model.into()).collect())
    }
    
    pub async fn get_quest_by_alias(&self, alias: &str) -> Result<Option<Quest>, DbErr> {
        let quest_alias = QuestAliasEntity::find()
            .filter(quest_alias::Column::Alias.eq(alias))
            .one(&self.conn)
            .await?;
            
        if let Some(qa) = quest_alias {
            let quest = QuestEntity::find()
                .filter(quest::Column::TargetId.eq(qa.target_id))
                .one(&self.conn)
                .await?;
                
            return Ok(quest.map(|q| q.into()));
        }
        
        Ok(None)
    }
    
    pub async fn get_quest_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, DbErr> {
        let quest = QuestEntity::find()
            .filter(quest::Column::TargetId.eq(target_id))
            .one(&self.conn)
            .await?;
            
        Ok(quest.map(|q| q.into()))
    }
}