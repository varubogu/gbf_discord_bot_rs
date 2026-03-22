use crate::events::helpers::resolve_guild_locale;
use crate::events::interactions::help_navigator::{
    HELP_NAV_JUMP_CUSTOM_ID, HELP_NAV_TO_INDEX_CUSTOM_ID, HelpPage, build_help_view,
    build_visible_pages, parse_help_nav_custom_id, resolve_next_page,
};
use crate::events::permission::resolve_bot_control_for_interaction;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::env;
use tracing::{debug, error, info, warn};

/// ヘルプ画面のページ遷移ボタンを処理する
pub async fn handle_help_navigation(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = interaction.data.custom_id.as_str();
    let selected_jump_page = if custom_id == HELP_NAV_JUMP_CUSTOM_ID {
        extract_jump_page(interaction)
    } else {
        None
    };
    let parsed_nav_input = parse_help_nav_custom_id(custom_id);
    let move_to_index = custom_id == HELP_NAV_TO_INDEX_CUSTOM_ID;

    if !move_to_index && selected_jump_page.is_none() && parsed_nav_input.is_none() {
        warn!(
            custom_id = %interaction.data.custom_id,
            "ヘルプ遷移カスタムIDの解析に失敗したため処理を中断します"
        );
        interaction.defer(&ctx.http).await.map_err(|e| {
            error!(error = %e, "不正なヘルプ遷移でdeferに失敗しました");
            AppError::Discord(Box::new(e))
        })?;
        return Ok(());
    };

    let guild_id = interaction.guild_id.map(|id| id.get() as i64);
    let locale = resolve_guild_locale(&data.app_state, guild_id).await;
    let has_bot_control = resolve_bot_control_for_interaction(ctx, interaction).await;
    let is_admin_server = resolve_is_admin_server(interaction.guild_id.map(|id| id.get()));
    let visible_pages = build_visible_pages(has_bot_control, is_admin_server);
    let next_page = if move_to_index {
        HelpPage::Index
    } else if let Some(page) = selected_jump_page {
        page
    } else if let Some(nav_input) = parsed_nav_input {
        resolve_next_page(&visible_pages, nav_input.current_page, nav_input.direction)
    } else {
        warn!(
            custom_id = %interaction.data.custom_id,
            "ヘルプ遷移の入力が不正なためIndexにフォールバックします"
        );
        HelpPage::Index
    };

    let view = build_help_view(
        &data.app_state,
        guild_id,
        &locale,
        next_page,
        &visible_pages,
    )
    .await;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(view.embed)
                    .components(view.components),
            ),
        )
        .await
        .map_err(|e| {
            error!(error = %e, "ヘルプ遷移時のレスポンス更新に失敗しました");
            AppError::Discord(Box::new(e))
        })?;

    info!(
        user_id = %interaction.user.id,
        guild_id = ?interaction.guild_id.map(|id| id.get()),
        "ヘルプ画面の遷移を処理しました"
    );

    Ok(())
}

fn extract_jump_page(interaction: &ComponentInteraction) -> Option<HelpPage> {
    let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind else {
        return None;
    };

    values.first().and_then(|value| HelpPage::from_id(value))
}

fn resolve_is_admin_server(guild_id: Option<u64>) -> bool {
    let Some(guild_id) = guild_id else {
        debug!("ギルド外のため管理サーバー判定はfalseを返します");
        return false;
    };

    let admin_server_id = env::var("BOT_ADMIN_SERVER_ID").unwrap_or_else(|_| String::new());
    if admin_server_id.is_empty() {
        return false;
    }

    guild_id.to_string() == admin_server_id
}
