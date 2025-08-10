use std::sync::Arc;
use serenity::all::{Context, Reaction, Message};
use tracing::{error, info};

use crate::services::database::Database;
use crate::utils::discord_helper::{get_reaction_users, update_embed_with_participants, get_unique_reaction_users};

pub struct ReactionHandler {
    db: Arc<Database>,
}

impl ReactionHandler {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    pub async fn handle_reaction_add(&self, ctx: Context, reaction: Reaction) -> Result<(), String> {
        // Skip reactions from the bot itself
        if reaction.user_id.unwrap() == ctx.cache.current_user().id {
            return Ok(());
        }
        
        self.handle_reaction(ctx, reaction).await
    }
    
    pub async fn handle_reaction_remove(&self, ctx: Context, reaction: Reaction) -> Result<(), String> {
        // Skip reactions from the bot itself
        if reaction.user_id.unwrap() == ctx.cache.current_user().id {
            return Ok(());
        }
        
        self.handle_reaction(ctx, reaction).await
    }
    
    async fn handle_reaction(&self, ctx: Context, reaction: Reaction) -> Result<(), String> {
        let guild_id = match reaction.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };
        
        let channel_id = reaction.channel_id;
        let message_id = reaction.message_id;
        
        // Check if this is a recruitment message
        let recruitment = match self.db.get_battle_recruitment(
            guild_id.get() as i64,
            channel_id.get() as i64,
            message_id.get() as i64,
        ).await {
            Ok(Some(recruitment)) => recruitment,
            Ok(None) => return Ok(()), // Not a recruitment message
            Err(e) => {
                error!("Error fetching recruitment: {:?}", e);
                return Err(format!("Database error: {}", e));
            }
        };
        
        // Get the message
        let message = match channel_id.message(&ctx.http, message_id).await {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error fetching message: {:?}", e);
                return Err(format!("Failed to fetch message: {}", e));
            }
        };
        
        // Check if the message is from our bot
        if message.author.id != ctx.cache.current_user().id {
            return Ok(());
        }
        
        // Get all reactions and users
        let reactions = get_reaction_users(&ctx, &message).await?;
        
        // Update the embed with participants
        update_embed_with_participants(&ctx, &message, reactions.clone()).await?;
        
        // Check if all required participants have joined
        self.check_recruitment_complete(&ctx, &message, &recruitment, reactions).await?;
        
        Ok(())
    }
    
    async fn check_recruitment_complete(
        &self,
        ctx: &Context,
        message: &Message,
        recruitment: &crate::services::database::BattleRecruitment,
        reactions: std::collections::HashMap<String, Vec<serenity::all::User>>,
    ) -> Result<(), String> {
        // Get the quest
        let quest = match self.db.get_quest_by_target_id(recruitment.target_id).await {
            Ok(Some(quest)) => quest,
            Ok(None) => {
                error!("Quest not found for target_id: {}", recruitment.target_id);
                return Ok(());
            },
            Err(e) => {
                error!("Error fetching quest: {:?}", e);
                return Err(format!("Database error: {}", e));
            }
        };
        
        // Count unique users across all reactions
        let unique_users = get_unique_reaction_users(ctx, message).await?;
        
        // Check if recruitment is complete (assuming recruit_count is 6 for now)
        // In a real implementation, you'd get this from the quest data
        let recruit_count = 6; // Default value
        
        if unique_users.len() >= recruit_count as usize {
            // Check if we've already sent a completion message
            if let Ok(Some(true)) = self.db.has_recruitment_end_message(recruitment.id).await {
                return Ok(());
            }
            
            // Get the completion message text
            let message_text = match self.db.get_message_text(
                recruitment.guild_id,
                "MSG00032",
            ).await {
                Ok(Some(msg)) => msg.message_jp,
                Ok(None) => "募集が完了しました！".to_string(),
                Err(e) => {
                    error!("Error fetching message text: {:?}", e);
                    "募集が完了しました！".to_string()
                }
            };
            
            // Create mentions for all participants
            let mentions = unique_users.iter()
                .map(|id| format!("<@{}>", id))
                .collect::<Vec<_>>()
                .join(" ");
            
            // Send completion message
            let content = format!("{}\n{}", mentions, message_text);
            let reply = match message.channel_id.say(&ctx.http, content).await {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error sending completion message: {:?}", e);
                    return Err(format!("Failed to send completion message: {}", e));
                }
            };
            
            // Update the recruitment record
            if let Err(e) = self.db.set_recruitment_end_message(
                recruitment.id,
                reply.id.get() as i64,
            ).await {
                error!("Error updating recruitment record: {:?}", e);
            }
        }
        
        Ok(())
    }
}