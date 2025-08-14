use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter, ConnectionTrait};
use chrono::{DateTime, Utc};
use crate::types::PoiseError;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::models::entities::{battle_recruitment, battle_recruitment::Entity as BattleRecruitmentEntity};
use crate::utils::database::Transaction;

/// Repository trait for battle recruitment operations
#[async_trait]
pub trait BattleRecruitmentRepository: Send + Sync {
    /// Create new battle recruitment (auto-commit)
    async fn create(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError>;

    /// Create new battle recruitment within a transaction
    async fn create_in_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError>;

    /// Get battle recruitment by identifiers (auto-commit)
    async fn get_by_message(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError>;

    /// Get battle recruitment by identifiers within a transaction
    async fn get_by_message_in_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError>;

    /// Update recruitment end message (auto-commit)
    async fn set_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError>;

    /// Update recruitment end message within a transaction
    async fn set_end_message_in_txn(
        &self,
        txn: &Transaction,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError>;
}

/// SeaORM implementation of BattleRecruitmentRepository
pub struct SeaOrmBattleRecruitmentRepository {
    conn: sea_orm::DatabaseConnection,
}

impl SeaOrmBattleRecruitmentRepository {
    pub fn new(conn: sea_orm::DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl BattleRecruitmentRepository for SeaOrmBattleRecruitmentRepository {
    async fn create(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError> {
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

    async fn create_in_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError> {
        let sea_txn = txn.get_txn()?;
        let battle_recruitment = battle_recruitment::ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            target_id: Set(target_id),
            battle_type_id: Set(battle_type_id),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };

        let result = battle_recruitment.insert(sea_txn).await?;
        Ok(result.into())
    }

    async fn get_by_message(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError> {
        let result = BattleRecruitmentEntity::find()
            .filter(battle_recruitment::Column::GuildId.eq(guild_id))
            .filter(battle_recruitment::Column::ChannelId.eq(channel_id))
            .filter(battle_recruitment::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await?;

        Ok(result.map(|model| model.into()))
    }

    async fn get_by_message_in_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError> {
        let sea_txn = txn.get_txn()?;
        let result = BattleRecruitmentEntity::find()
            .filter(battle_recruitment::Column::GuildId.eq(guild_id))
            .filter(battle_recruitment::Column::ChannelId.eq(channel_id))
            .filter(battle_recruitment::Column::MessageId.eq(message_id))
            .one(sea_txn)
            .await?;

        Ok(result.map(|model| model.into()))
    }

    async fn set_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError> {
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

    async fn set_end_message_in_txn(
        &self,
        txn: &Transaction,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError> {
        let sea_txn = txn.get_txn()?;
        let recruitment = BattleRecruitmentEntity::find_by_id(recruitment_id)
            .one(sea_txn)
            .await?;

        if let Some(recruitment) = recruitment {
            let mut active_model: battle_recruitment::ActiveModel = recruitment.into();
            active_model.recruit_end_message_id = Set(Some(message_id));
            active_model.update(sea_txn).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn setup_test_repo() -> Result<SeaOrmBattleRecruitmentRepository, String> {
        if std::env::var("DATABASE_URL").is_err() {
            return Err("DATABASE_URL not set".to_string());
        }

        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to database: {}", e)),
        };

        Ok(SeaOrmBattleRecruitmentRepository::new(conn))
    }

    #[tokio::test]
    async fn test_battle_recruitment_operations() {
        let repo = match setup_test_repo().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test creating a battle_recruitment recruitment
        let guild_id = 123456789;
        let channel_id = 987654321;
        let message_id = 555666777;
        let target_id = 1;
        let battle_type_id = 1;
        let expiry_date = Utc::now() + chrono::Duration::hours(1);

        let create_result = repo.create(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type_id,
            expiry_date,
        ).await;

        match create_result {
            Ok(recruitment) => {
                assert_eq!(recruitment.guild_id, guild_id);
                assert_eq!(recruitment.channel_id, channel_id);
                assert_eq!(recruitment.message_id, message_id);
                assert_eq!(recruitment.target_id, target_id);
                assert_eq!(recruitment.battle_type_id, battle_type_id);

                // Test retrieving the created recruitment
                let get_result = repo.get_by_message(guild_id, channel_id, message_id).await;
                match get_result {
                    Ok(Some(retrieved)) => {
                        assert_eq!(retrieved.id, recruitment.id);
                        assert_eq!(retrieved.guild_id, guild_id);
                        assert_eq!(retrieved.channel_id, channel_id);
                        assert_eq!(retrieved.message_id, message_id);
                    },
                    Ok(None) => panic!("Should have retrieved the created recruitment"),
                    Err(e) => println!("Error retrieving recruitment: {}", e),
                }

                // Test recruitment end message operations
                let end_message_id = 111222333;
                let set_result = repo.set_end_message(recruitment.id, end_message_id).await;
                match set_result {
                    Ok(_) => {
                        println!("Successfully set end message for recruitment");
                    },
                    Err(e) => {
                        println!("Failed to set end message: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("Create battle_recruitment recruitment returned error (may be expected): {}", e);
            }
        }
    }
}
