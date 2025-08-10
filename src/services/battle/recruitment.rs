use std::sync::Arc;
use chrono::{Local, Timelike};
use serenity::all::{
    Context, CreateEmbed, Message,
};
use tracing::{error, info};

use crate::services::database::{Database, Quest};
use super::battle_type::BattleType;

pub struct RecruitmentService {
    db: Arc<Database>,
}

impl RecruitmentService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    pub async fn default_expiry_date() -> chrono::DateTime<Local> {
        let now = Local::now();
        now.with_hour(21)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }
    
    pub async fn parse_event_date(date_str: &str) -> Result<chrono::DateTime<Local>, String> {
        // Simple implementation - in a real app, you'd want more robust parsing
        if date_str == "今日 21:00" {
            return Ok(Self::default_expiry_date().await);
        }
        
        // For now, just return the default
        Ok(Self::default_expiry_date().await)
    }
    
    pub async fn get_quest_by_alias(&self, alias: &str) -> Result<Option<Quest>, String> {
        match self.db.get_quest_by_alias(alias).await {
            Ok(quest) => Ok(quest),
            Err(e) => {
                error!("Database error when getting quest by alias: {:?}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }
    
    pub async fn create_recruitment_message(
        &self,
        quest_name: &str,
        battle_type: BattleType,
        event_date: chrono::DateTime<Local>,
    ) -> (String, CreateEmbed) {
        // Create message
        let message_text = if battle_type == BattleType::AllElement {
            format!("{}の参加者を募集します！", quest_name)
        } else {
            format!("{}の{}参加者を募集します！", quest_name, battle_type.name())
        };
        
        let message_with_date = format!(
            "{}\n開催日時：{}", 
            message_text, 
            event_date.format("%m/%d %H:%M")
        );
        
        // Create embed
        let embed = CreateEmbed::new()
            .title("参加者一覧")
            .description("現在参加者はいません。");
            
        (message_with_date, embed)
    }
    
    pub async fn register_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type: BattleType,
        event_date: chrono::DateTime<Local>,
    ) -> Result<(), String> {
        if let Err(e) = self.db.create_battle_recruitment(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type as i32,
            event_date.with_timezone(&chrono::Utc),
        ).await {
            error!("Error registering recruitment in database: {:?}", e);
            return Err(format!("Failed to register recruitment: {}", e));
        }
        
        Ok(())
    }
    
    pub async fn add_reactions(
        &self,
        ctx: &Context,
        message: &Message,
        battle_type: BattleType,
    ) -> Result<(), String> {
        for reaction in battle_type.reactions() {
            if let Err(e) = message.react(&ctx.http, reaction).await {
                error!("Error adding reaction: {:?}", e);
                // Continue with other reactions even if one fails
            }
        }
        
        Ok(())
    }
}