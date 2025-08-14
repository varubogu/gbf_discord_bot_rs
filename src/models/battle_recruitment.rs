use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, DbErr};

use crate::models::entities::{battle_recruitment, battle_recruitment::Entity as BattleRecruitmentEntity};
use crate::models::database::Database;

/// Battle recruitment domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecruitment {
    pub id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub target_id: i32,
    pub battle_type_id: i32,
    pub expiry_date: DateTime<Utc>,
    pub recruit_end_message_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<battle_recruitment::Model> for BattleRecruitment {
    fn from(model: battle_recruitment::Model) -> Self {
        Self {
            id: model.id,
            guild_id: model.guild_id,
            channel_id: model.channel_id,
            message_id: model.message_id,
            target_id: model.target_id,
            battle_type_id: model.battle_type_id,
            expiry_date: model.expiry_date,
            recruit_end_message_id: model.recruit_end_message_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn create_battle_recruitment(
        &self, 
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<BattleRecruitment, DbErr> {
        let battle_recruitment = battle_recruitment::ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            target_id: Set(target_id),
            battle_type_id: Set(battle_type_id),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };

        let result = battle_recruitment.insert(&self.conn).await?;
        Ok(result.into())
    }
    
    pub async fn get_battle_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, DbErr> {
        let result = BattleRecruitmentEntity::find()
            .filter(battle_recruitment::Column::GuildId.eq(guild_id))
            .filter(battle_recruitment::Column::ChannelId.eq(channel_id))
            .filter(battle_recruitment::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await?;
            
        Ok(result.map(|model| model.into()))
    }
    
    pub async fn has_recruitment_end_message(
        &self,
        recruitment_id: i32,
    ) -> Result<Option<bool>, DbErr> {
        let result = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.conn)
            .await?;
            
        Ok(result.map(|model| model.recruit_end_message_id.is_some()))
    }
    
    pub async fn set_recruitment_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), DbErr> {
        let recruitment = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.conn)
            .await?;
            
        if let Some(recruitment) = recruitment {
            let mut active_model: battle_recruitment::ActiveModel = recruitment.into();
            active_model.recruit_end_message_id = Set(Some(message_id));
            active_model.update(&self.conn).await?;
        }
        
        Ok(())
    }
}