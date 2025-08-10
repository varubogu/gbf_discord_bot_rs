use std::sync::Arc;
use tracing::info;

// Import the models Database struct
use crate::models;
// Import the old model structs for compatibility
use super::models::{Quest, QuestAlias, BattleRecruitment, MessageText, Environment};

pub struct Database {
    inner: Arc<models::Database>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        info!("Creating database connection...");
        let inner = match models::Database::new().await {
            Ok(db) => Arc::new(db),
            Err(e) => return Err(sqlx::Error::Protocol(format!("Failed to connect to database: {}", e))),
        };
            
        info!("Connected to database");
        Ok(Self { inner })
    }
    
    pub async fn get_quests(&self) -> Result<Vec<Quest>, sqlx::Error> {
        match self.inner.get_quests().await {
            Ok(quests) => Ok(quests.into_iter().map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_quest_aliases(&self) -> Result<Vec<QuestAlias>, sqlx::Error> {
        match self.inner.get_quest_aliases().await {
            Ok(aliases) => Ok(aliases.into_iter().map(|a| QuestAlias {
                id: a.id,
                target_id: a.target_id,
                alias: a.alias,
                created_at: a.created_at,
                updated_at: a.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_quest_by_alias(&self, alias: &str) -> Result<Option<Quest>, sqlx::Error> {
        match self.inner.get_quest_by_alias(alias).await {
            Ok(quest) => Ok(quest.map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_quest_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, sqlx::Error> {
        match self.inner.get_quest_by_target_id(target_id).await {
            Ok(quest) => Ok(quest.map(|q| Quest {
                id: q.id,
                target_id: q.target_id,
                quest_name: q.quest_name,
                default_battle_type: q.default_battle_type,
                created_at: q.created_at,
                updated_at: q.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn create_battle_recruitment(
        &self, 
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<BattleRecruitment, sqlx::Error> {
        match self.inner.create_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type_id,
            expiry_date,
        ).await {
            Ok(recruitment) => Ok(BattleRecruitment {
                id: recruitment.id,
                guild_id: recruitment.guild_id,
                channel_id: recruitment.channel_id,
                message_id: recruitment.message_id,
                target_id: recruitment.target_id,
                battle_type_id: recruitment.battle_type_id,
                expiry_date: recruitment.expiry_date,
                recruit_end_message_id: recruitment.recruit_end_message_id,
                created_at: recruitment.created_at,
                updated_at: recruitment.updated_at,
            }),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_battle_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, sqlx::Error> {
        match self.inner.get_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
        ).await {
            Ok(recruitment) => Ok(recruitment.map(|r| BattleRecruitment {
                id: r.id,
                guild_id: r.guild_id,
                channel_id: r.channel_id,
                message_id: r.message_id,
                target_id: r.target_id,
                battle_type_id: r.battle_type_id,
                expiry_date: r.expiry_date,
                recruit_end_message_id: r.recruit_end_message_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn has_recruitment_end_message(
        &self,
        recruitment_id: i32,
    ) -> Result<Option<bool>, sqlx::Error> {
        match self.inner.has_recruitment_end_message(recruitment_id).await {
            Ok(has_message) => Ok(has_message),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn set_recruitment_end_message(
        &self,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), sqlx::Error> {
        match self.inner.set_recruitment_end_message(recruitment_id, message_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_message_text(
        &self,
        guild_id: i64,
        message_id: &str,
    ) -> Result<Option<MessageText>, sqlx::Error> {
        match self.inner.get_message_text(guild_id, message_id).await {
            Ok(message_text) => Ok(message_text.map(|mt| MessageText {
                id: mt.id,
                guild_id: mt.guild_id,
                message_id: mt.message_id,
                message_jp: mt.message_jp,
                message_en: mt.message_en,
                created_at: mt.created_at,
                updated_at: mt.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_environments(&self) -> Result<Vec<Environment>, sqlx::Error> {
        match self.inner.get_environments().await {
            Ok(environments) => Ok(environments.into_iter().map(|env| Environment {
                id: env.id,
                key: env.key,
                value: env.value,
                created_at: env.created_at,
                updated_at: env.updated_at,
            }).collect()),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn get_environment(&self, key: &str) -> Result<Option<Environment>, sqlx::Error> {
        match self.inner.get_environment(key).await {
            Ok(environment) => Ok(environment.map(|env| Environment {
                id: env.id,
                key: env.key,
                value: env.value,
                created_at: env.created_at,
                updated_at: env.updated_at,
            })),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
    
    pub async fn set_environment(&self, key: &str, value: &str) -> Result<Environment, sqlx::Error> {
        match self.inner.set_environment(key, value).await {
            Ok(environment) => Ok(Environment {
                id: environment.id,
                key: environment.key,
                value: environment.value,
                created_at: environment.created_at,
                updated_at: environment.updated_at,
            }),
            Err(e) => Err(sqlx::Error::Protocol(format!("Database error: {}", e))),
        }
    }
}