use poise::serenity_prelude::all::{
    Context, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateEmbed, CreateEmbedFooter,
};
use tracing::error;

pub async fn handle_help_command(
    ctx: &Context, 
    command: &CommandInteraction,
) -> Result<(), String> {
    // Create an embed with help information
    let embed = CreateEmbed::new()
        .title("GBF Discord Bot Help")
        .description("This bot helps manage Granblue Fantasy game activities in Discord servers.")
        .field(
            "Commands",
            "Here are the available commands:",
            false
        )
        .field(
            "/recruit",
            "Create a battle recruitment with reactions for different elements.\n\
            Usage: `/recruit quest:<quest_name> [battle_type:<type>] [event_date:<date>]`",
            false
        )
        .field(
            "/environ_load",
            "Reload environment variables from the database.\n\
            Usage: `/environ_load`\n\
            Note: Requires the 'gbf_bot_control' role.",
            false
        )
        .field(
            "/help",
            "Show this help message.\n\
            Usage: `/help`",
            false
        )
        .footer(CreateEmbedFooter::new("GBF Discord Bot - Rust Edition"));
    
    // Send the response
    if let Err(e) = command.create_response(&ctx.http, 
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .add_embed(embed)
                .ephemeral(true)
        )
    ).await {
        error!("Error sending help response: {:?}", e);
        return Err(format!("Failed to send help response: {}", e));
    }
    
    Ok(())
}