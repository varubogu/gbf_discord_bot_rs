use std::sync::Arc;

use poise::serenity_prelude::all::{
    Context, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use tracing::error;

use crate::services::database::Database;
use crate::services::environment::load_environment_from_database;
use crate::services::permission::has_bot_control_permission;

pub async fn handle_environ_load_command(
    ctx: &Context, 
    command: &CommandInteraction,
    db: Arc<Database>
) -> Result<(), String> {
    // Check if user has the required role
    let member = match command.member.as_ref() {
        Some(member) => member,
        None => {
            return Err("Command must be used in a guild".to_string());
        }
    };
    
    // Use the permission service to check if the user has the required role
    let has_permission = has_bot_control_permission(ctx, member).await;
    
    if !has_permission {
        if let Err(e) = command.create_response(&ctx.http, 
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("You don't have permission to use this command")
                    .ephemeral(true)
            )
        ).await {
            error!("Error sending permission error: {:?}", e);
        }
        return Err("User doesn't have permission".to_string());
    }
    
    // Send initial response
    if let Err(e) = command.create_response(&ctx.http, 
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("環境変数読み込み中...")
                .ephemeral(true)
        )
    ).await {
        error!("Error sending initial response: {:?}", e);
        return Err(format!("Failed to send initial response: {}", e));
    }
    
    // Use the service function to load environment variables from database and update ENV
    match load_environment_from_database(db).await {
        Ok(_) => {
            // Send success message
            if let Err(e) = command.edit_response(&ctx.http, 
                EditInteractionResponse::new()
                    .content("環境変数読み込み完了")
            ).await {
                error!("Error sending success message: {:?}", e);
                return Err(format!("Failed to send success message: {}", e));
            }
            
            Ok(())
        },
        Err(e) => {
            error!("Error loading environment from database: {:?}", e);
            
            // Send error message
            if let Err(e) = command.edit_response(&ctx.http, 
                EditInteractionResponse::new()
                    .content("環境変数読み込み失敗")
            ).await {
                error!("Error sending error message: {:?}", e);
            }
            
            Err(format!("Failed to load environment from database: {}", e))
        }
    }
}