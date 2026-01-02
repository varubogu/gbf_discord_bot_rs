use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

#[poise::command(
    slash_command,
    name_localized("ja", "ヘルプ"),
    description_localized("ja", "ヘルプを表示します"),
    ephemeral = true
)]
pub async fn help(ctx: PoiseContext<'_>) -> Result<()> {
    // Create an embed with help information
    let embed = CreateEmbed::new()
        .title("GBF Discord Bot Help")
        .description("This bot helps manage Granblue Fantasy game activities in Discord servers.")
        .field("Commands", "Here are the available commands:", false)
        .field(
            "/recruit",
            "Create a battle_recruitment recruitment with reactions for different elements.\n\
            Usage: `/recruit quest:<quest_name> event_date:<date> [battle_style:<type>]`",
            false,
        )
        .field(
            "/environ_load",
            "Reload environment variables from the database.\n\
            Usage: `/environ_load`\n\
            Note: Requires the 'gbf_bot_control' role.",
            false,
        )
        .field(
            "/gspread_load",
            "Load data from server-specific Google Spreadsheet.\n\
            Usage: `/gspread_load`\n\
            Note: Requires the 'gbf_bot_control' role.",
            false,
        )
        .field(
            "/gspread_push",
            "Push data from database to server-specific Google Spreadsheet.\n\
            Usage: `/gspread_push`\n\
            Note: Requires the 'gbf_bot_control' role.",
            false,
        )
        .field(
            "/help",
            "Show this help message.\n\
            Usage: `/help`",
            false,
        )
        .footer(CreateEmbedFooter::new("GBF Discord Bot - Rust Edition"));

    // Send the response using Poise's reply method
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
