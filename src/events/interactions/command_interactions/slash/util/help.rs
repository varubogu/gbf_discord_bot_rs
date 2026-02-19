use crate::events::helpers::get_message_or_fallback_from_context;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;

#[poise::command(
    slash_command,
    name_localized("ja", "ヘルプ"),
    description_localized("ja", "ヘルプを表示します"),
    ephemeral = true
)]
pub async fn help(ctx: PoiseContext<'_>) -> Result<()> {
    let title = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedTitle,
        HashMap::new(),
        "GBF Discord Bot Help",
    )
    .await;
    let description = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedDescription,
        HashMap::new(),
        "This bot helps manage Granblue Fantasy game activities in Discord servers.",
    )
    .await;
    let commands_field_title = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedCommandsFieldTitle,
        HashMap::new(),
        "Commands",
    )
    .await;
    let commands_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedCommandsFieldValue,
        HashMap::new(),
        "Here are the available commands:",
    )
    .await;
    let recruit_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedRecruitFieldValue,
        HashMap::new(),
        "Create a battle_recruitment recruitment with reactions for different elements.\n\
        Usage: `/recruit quest:<quest_name> event_date:<date> [battle_style:<type>]`",
    )
    .await;
    let environ_load_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedEnvironLoadFieldValue,
        HashMap::new(),
        "Reload environment variables from the database.\n\
        Usage: `/environ_load`\n\
        Note: Requires the 'gbf_bot_control' role.",
    )
    .await;
    let gspread_load_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedGspreadLoadFieldValue,
        HashMap::new(),
        "Load data from server-specific Google Spreadsheet.\n\
        Usage: `/gspread_load`\n\
        Note: Requires the 'gbf_bot_control' role.",
    )
    .await;
    let gspread_push_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedGspreadPushFieldValue,
        HashMap::new(),
        "Push data from database to server-specific Google Spreadsheet.\n\
        Usage: `/gspread_push`\n\
        Note: Requires the 'gbf_bot_control' role.",
    )
    .await;
    let help_field_value = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedHelpFieldValue,
        HashMap::new(),
        "Show this help message.\n\
        Usage: `/help`",
    )
    .await;
    let footer = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::HelpEmbedFooter,
        HashMap::new(),
        "GBF Discord Bot - Rust Edition",
    )
    .await;

    // Create an embed with help information
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .field(commands_field_title, commands_field_value, false)
        .field("/recruit", recruit_field_value, false)
        .field("/environ_load", environ_load_field_value, false)
        .field("/gspread_load", gspread_load_field_value, false)
        .field("/gspread_push", gspread_push_field_value, false)
        .field("/help", help_field_value, false)
        .footer(CreateEmbedFooter::new(footer));

    // Send the response using Poise's reply method
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
