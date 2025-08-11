use std::sync::Arc;
use poise::serenity_prelude::all::{
    Context, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::{error, info};

use crate::services::battle::{BattleType, RecruitmentService};
use crate::services::database::Database;

pub async fn handle_recruit_command(
    ctx: &Context, 
    command: &CommandInteraction,
    db: Arc<Database>
) -> Result<(), String> {
    // Create recruitment service
    let recruitment_service = RecruitmentService::new(db);
    
    // Extract command options
    let options = &command.data.options;
    
    // Get quest alias
    let quest_alias = options.iter()
        .find(|opt| opt.name == "quest")
        .and_then(|opt| opt.value.as_str())
        .ok_or("Quest option is required")?;
        
    // Get battle type
    let battle_type_value = options.iter()
        .find(|opt| opt.name == "battle_type")
        .and_then(|opt| opt.value.as_i64())
        .unwrap_or(0) as i32;
        
    let battle_type = BattleType::from_value(battle_type_value)
        .unwrap_or(BattleType::Default);
        
    // Get event date
    let event_date_str = options.iter()
        .find(|opt| opt.name == "event_date")
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("今日 21:00");
    
    let event_date = RecruitmentService::parse_event_date(event_date_str).await?;
    
    // Try to get quest from database
    let quest = match recruitment_service.get_quest_by_alias(quest_alias).await {
        Ok(Some(quest)) => quest,
        Ok(None) => {
            // If quest not found in database, just use the alias as the name
            info!("Quest not found in database: {}", quest_alias);
            return handle_recruit_with_name(&recruitment_service, ctx, command, quest_alias, battle_type, event_date).await;
        },
        Err(e) => {
            error!("Database error: {:?}", e);
            return handle_recruit_with_name(&recruitment_service, ctx, command, quest_alias, battle_type, event_date).await;
        }
    };
    
    // Determine actual battle type based on quest default if needed
    let actual_battle_type = if battle_type == BattleType::Default {
        BattleType::from_value(quest.default_battle_type)
            .unwrap_or(BattleType::Default)
    } else {
        battle_type
    };
    
    // Create message
    let (message_content, embed) = recruitment_service.create_recruitment_message(
        &quest.quest_name,
        actual_battle_type,
        event_date
    ).await;
    
    // Send response
    if let Err(e) = command.create_response(&ctx.http, 
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(message_content)
                .embed(embed)
        )
    ).await {
        error!("Error sending response: {:?}", e);
        return Err(format!("Failed to send response: {}", e));
    }
    
    // Get the message we just sent
    let response = match command.get_response(&ctx.http).await {
        Ok(response) => response,
        Err(e) => {
            error!("Error getting response: {:?}", e);
            return Err(format!("Failed to get response: {}", e));
        }
    };
    
    // Add reactions
    recruitment_service.add_reactions(ctx, &response, actual_battle_type).await?;
    
    // Register in database
    recruitment_service.register_recruitment(
        command.guild_id.unwrap_or_default().get() as i64,
        response.channel_id.get() as i64,
        response.id.get() as i64,
        quest.target_id,
        actual_battle_type,
        event_date,
    ).await?;
    
    Ok(())
}

// Fallback function when quest is not found in database
async fn handle_recruit_with_name(
    recruitment_service: &RecruitmentService,
    ctx: &Context,
    command: &CommandInteraction,
    quest_name: &str,
    battle_type: BattleType,
    event_date: chrono::DateTime<chrono::Local>,
) -> Result<(), String> {
    // Create message
    let (message_content, embed) = recruitment_service.create_recruitment_message(
        quest_name,
        battle_type,
        event_date
    ).await;
    
    // Send response
    if let Err(e) = command.create_response(&ctx.http, 
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(message_content)
                .embed(embed)
        )
    ).await {
        error!("Error sending response: {:?}", e);
        return Err(format!("Failed to send response: {}", e));
    }
    
    // Get the message we just sent
    match command.get_response(&ctx.http).await {
        Ok(response) => {
            // Add reactions
            recruitment_service.add_reactions(ctx, &response, battle_type).await?;
            Ok(())
        },
        Err(e) => {
            error!("Error getting response: {:?}", e);
            Err(format!("Failed to get response: {}", e))
        }
    }
}