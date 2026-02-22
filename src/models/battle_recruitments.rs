use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::infrastructure::database::session::DatabaseSession as Database;
use crate::models::entities::worker::battle_recruitments::{
    self, Entity as BattleRecruitmentEntity,
};

/// Battle recruitment domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecruitments {
    pub id: i32,
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_at: DateTime<Utc>,
    pub is_recruiting: bool,
    pub is_canceled: bool,
    pub recruit_end_message_id: Option<u64>,
    pub full_notification_sent: bool,
    /// 募集作成者（ホスト）のDiscordユーザーID。0は不明（旧データ）を表す。
    pub host_discord_user_id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<battle_recruitments::Model> for BattleRecruitments {
    fn from(model: battle_recruitments::Model) -> Self {
        Self {
            id: model.id,
            guild_id: model.guild_id as u64,     // i64 → u64に変換
            channel_id: model.channel_id as u64, // i64 → u64に変換
            message_id: model.message_id as u64, // i64 → u64に変換
            quest_id: model.quest_id,
            battle_style_id: model.battle_style_id,
            quest_start_at: model.quest_start_at,
            is_recruiting: model.is_recruiting,
            is_canceled: model.is_canceled,
            recruit_end_message_id: model.recruit_end_message_id.map(|id| id as u64), // i64 → u64に変換
            full_notification_sent: model.full_notification_sent,
            host_discord_user_id: model.host_discord_user_id as u64, // i64 → u64に変換
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
        battle_style_id: i32,
        quest_start_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<BattleRecruitments, DbErr> {
        use sea_orm::ActiveModelBehavior;
        let mut battle_recruitment = battle_recruitments::ActiveModel::new();
        battle_recruitment.guild_id = Set(guild_id as i64); // u64 → i64に変換
        battle_recruitment.channel_id = Set(channel_id as i64); // u64 → i64に変換
        battle_recruitment.message_id = Set(message_id as i64); // u64 → i64に変換
        battle_recruitment.quest_id = Set(quest_id);
        battle_recruitment.battle_style_id = Set(battle_style_id);
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
            .filter(battle_recruitments::Column::GuildId.eq(guild_id as i64)) // u64 → i64に変換
            .filter(battle_recruitments::Column::ChannelId.eq(channel_id as i64)) // u64 → i64に変換
            .filter(battle_recruitments::Column::MessageId.eq(message_id as i64)) // u64 → i64に変換
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
            active_model.recruit_end_message_id = Set(Some(message_id as i64)); // u64 → i64に変換
            active_model.update(&self.conn).await?;
        }

        Ok(())
    }
}
