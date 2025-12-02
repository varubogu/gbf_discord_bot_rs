use crate::models::battle_recruitments::BattleRecruitments;
use crate::models::entities::battle_recruitments::{
    ActiveModel, Column, Entity as BattleRecruitmentEntity,
};
use crate::repository::BattleRecruitmentsRepository;
use crate::types::{AppError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, Set,
};

/// SeaORM を使用したバトル募集リポジトリの実装
#[derive(Debug)]
pub struct BattleRecruitmentsRepositoryImpl {
    connection: DatabaseConnection,
}

impl BattleRecruitmentsRepositoryImpl {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl BattleRecruitmentsRepository for BattleRecruitmentsRepositoryImpl {
    async fn create(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<BattleRecruitments> {
        let mut active_model = ActiveModel::new();
        active_model.guild_id = Set(guild_id as i64); // u64 → i64に変換
        active_model.channel_id = Set(channel_id as i64); // u64 → i64に変換
        active_model.message_id = Set(message_id as i64); // u64 → i64に変換
        active_model.quest_id = Set(quest_id);
        active_model.battle_style_id = Set(battle_style_id);
        active_model.quest_start_at = Set(quest_start_at);

        let result = active_model
            .insert(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(BattleRecruitments::from(result))
    }

    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<BattleRecruitments> {
        let mut active_model = ActiveModel::new();
        active_model.guild_id = Set(guild_id as i64); // u64 → i64に変換
        active_model.channel_id = Set(channel_id as i64); // u64 → i64に変換
        active_model.message_id = Set(message_id as i64); // u64 → i64に変換
        active_model.quest_id = Set(quest_id);
        active_model.battle_style_id = Set(battle_style_id);
        active_model.quest_start_at = Set(quest_start_at);

        let result = active_model
            .insert(txn)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(BattleRecruitments::from(result))
    }

    async fn get_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>> {
        let result = BattleRecruitmentEntity::find()
            .filter(Column::GuildId.eq(guild_id as i64)) // u64 → i64に変換
            .filter(Column::ChannelId.eq(channel_id as i64)) // u64 → i64に変換
            .filter(Column::MessageId.eq(message_id as i64)) // u64 → i64に変換
            .one(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(result.map(BattleRecruitments::from))
    }

    async fn get_by_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>> {
        let result = BattleRecruitmentEntity::find()
            .filter(Column::GuildId.eq(guild_id as i64)) // u64 → i64に変換
            .filter(Column::ChannelId.eq(channel_id as i64)) // u64 → i64に変換
            .filter(Column::MessageId.eq(message_id as i64)) // u64 → i64に変換
            .one(txn)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(result.map(BattleRecruitments::from))
    }

    async fn set_end_message(
        &self,
        recruitment_id: i32,
        message_id: poise::serenity_prelude::MessageId,
    ) -> Result<()> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::Business {
                message: "Recruitment not found".to_string(),
            })?
            .into();

        active_model.recruit_end_message_id = Set(Some(message_id.get() as i64)); // u64 → i64に変換
        active_model
            .update(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn set_canceled(
        &self,
        recruitment_id: i32,
        message_id: poise::serenity_prelude::MessageId,
    ) -> Result<()> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::Business {
                message: "Recruitment not found".to_string(),
            })?
            .into();

        active_model.recruit_end_message_id = Set(Some(message_id.get() as i64)); // u64 → i64に変換
        active_model.is_canceled = Set(true);
        active_model
            .update(&self.connection)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        message_id: poise::serenity_prelude::MessageId,
    ) -> Result<()> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(txn)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::Business {
                message: "Recruitment not found".to_string(),
            })?
            .into();

        active_model.recruit_end_message_id = Set(Some(message_id.get() as i64)); // u64 → i64に変換
        active_model.is_canceled = Set(true);
        active_model
            .update(txn)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn update_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<()> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(txn)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::Business {
                message: "Recruitment not found".to_string(),
            })?
            .into();

        active_model.quest_id = Set(quest_id);
        active_model.battle_style_id = Set(battle_style_id);
        active_model.quest_start_at = Set(quest_start_at);

        active_model
            .update(txn)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }
}
