use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::models::entities::{
    battle_recruitments, battle_recruitments::Entity as BattleRecruitmentEntity,
};
use crate::repository::database::db_compat::Database;

/// Battle recruitment domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecruitments {
    pub id: i32,
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub quest_id: i32,
    pub battle_type_id: i32,
    pub quest_start_at: DateTime<Utc>,
    pub is_recruiting: bool,
    pub is_canceled: bool,
    pub recruit_end_message_id: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<battle_recruitments::Model> for BattleRecruitments {
    fn from(model: battle_recruitments::Model) -> Self {
        Self {
            id: model.id,
            guild_id: model.guild_id,
            channel_id: model.channel_id,
            message_id: model.message_id,
            quest_id: model.quest_id,
            battle_type_id: model.battle_type_id,
            quest_start_at: model.quest_start_at,
            is_recruiting: model.is_recruiting,
            is_canceled: model.is_canceled,
            recruit_end_message_id: model.recruit_end_message_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn create_battle_recruitment(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        quest_id: i32,
        battle_type_id: i32,
        quest_start_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<BattleRecruitments, DbErr> {
        use sea_orm::ActiveModelBehavior;
        let mut battle_recruitment = battle_recruitments::ActiveModel::new();
        battle_recruitment.guild_id = Set(guild_id);
        battle_recruitment.channel_id = Set(channel_id);
        battle_recruitment.message_id = Set(message_id);
        battle_recruitment.quest_id = Set(quest_id);
        battle_recruitment.battle_type_id = Set(battle_type_id);
        battle_recruitment.quest_start_at = Set(quest_start_at);

        let result = battle_recruitment.insert(&self.conn).await?;
        Ok(result.into())
    }

    pub async fn get_battle_recruitment(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>, DbErr> {
        let result = BattleRecruitmentEntity::find()
            .filter(battle_recruitments::Column::GuildId.eq(guild_id))
            .filter(battle_recruitments::Column::ChannelId.eq(channel_id))
            .filter(battle_recruitments::Column::MessageId.eq(message_id))
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
        message_id: u64,
    ) -> Result<(), DbErr> {
        let recruitment = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.conn)
            .await?;

        if let Some(recruitment) = recruitment {
            let mut active_model: battle_recruitments::ActiveModel = recruitment.into();
            active_model.recruit_end_message_id = Set(Some(message_id));
            active_model.update(&self.conn).await?;
        }

        Ok(())
    }
}
