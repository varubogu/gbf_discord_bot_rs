use crate::infrastructure::database::transaction::Transaction;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::models::entities::battle_recruitment::{
    ActiveModel, Column, Entity as BattleRecruitmentEntity,
};
use crate::repository::BattleRecruitmentRepository;
use crate::types::PoiseError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// SeaORM を使用したバトル募集リポジトリの実装
pub struct BattleRecruitmentRepositoryImpl {
    connection: DatabaseConnection,
}

impl BattleRecruitmentRepositoryImpl {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl BattleRecruitmentRepository for BattleRecruitmentRepositoryImpl {
    async fn create(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError> {
        let active_model = ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            target_id: Set(target_id),
            battle_type_id: Set(battle_type_id),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };

        let result = active_model
            .insert(&self.connection)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to create battle recruitment: {}", e)))?;

        Ok(BattleRecruitment::from(result))
    }

    async fn get_by_message(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError> {
        let result = BattleRecruitmentEntity::find()
            .filter(Column::GuildId.eq(guild_id))
            .filter(Column::ChannelId.eq(channel_id))
            .filter(Column::MessageId.eq(message_id))
            .one(&self.connection)
            .await
            .map_err(|e| {
                PoiseError::from(format!(
                    "Failed to get battle recruitment by message: {}",
                    e
                ))
            })?;

        Ok(result.map(BattleRecruitment::from))
    }

    async fn set_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(&self.connection)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to find recruitment: {}", e)))?
            .ok_or_else(|| PoiseError::from("Recruitment not found".to_string()))?
            .into();

        active_model.recruit_end_message_id = Set(Some(message_id));
        active_model
            .update(&self.connection)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to update end message: {}", e)))?;

        Ok(())
    }

    async fn create_with_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError> {
        let active_model = ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            target_id: Set(target_id),
            battle_type_id: Set(battle_type_id),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };

        let result = active_model.insert(txn.get_txn()?).await.map_err(|e| {
            PoiseError::from(format!("Failed to create battle recruitment in txn: {}", e))
        })?;

        Ok(BattleRecruitment::from(result))
    }

    async fn get_by_message_with_txn(
        &self,
        txn: &Transaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitment>, PoiseError> {
        let result = BattleRecruitmentEntity::find()
            .filter(Column::GuildId.eq(guild_id))
            .filter(Column::ChannelId.eq(channel_id))
            .filter(Column::MessageId.eq(message_id))
            .one(txn.get_txn()?)
            .await
            .map_err(|e| {
                PoiseError::from(format!(
                    "Failed to get battle recruitment by message in txn: {}",
                    e
                ))
            })?;

        Ok(result.map(BattleRecruitment::from))
    }

    async fn set_end_message_with_txn(
        &self,
        txn: &Transaction,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError> {
        let mut active_model: ActiveModel = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(txn.get_txn()?)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to find recruitment in txn: {}", e)))?
            .ok_or_else(|| PoiseError::from("Recruitment not found".to_string()))?
            .into();

        active_model.recruit_end_message_id = Set(Some(message_id));
        active_model
            .update(txn.get_txn()?)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to update end message in txn: {}", e)))?;

        Ok(())
    }
}
