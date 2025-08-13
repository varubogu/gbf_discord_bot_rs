use crate::types::{PoiseContext, PoiseError};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

#[poise::command(
    slash_command,
    name_localized("ja", "ヘルプ"),
    description_localized("ja", "ヘルプを表示します"),
    ephemeral
)]
pub async fn help(
    ctx: PoiseContext<'_>,
) -> Result<(), PoiseError> {
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
            "Create a battle_recruitment recruitment with reactions for different elements.\n\
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
    
    // Send the response using Poise's reply method
    ctx.send(poise::CreateReply::default()
        .embed(embed)
        .ephemeral(true)
    ).await?;
    
    Ok(())
}