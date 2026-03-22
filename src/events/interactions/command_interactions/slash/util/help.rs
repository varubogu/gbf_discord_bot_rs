use crate::events::helpers::get_locale_from_context;
use crate::events::interactions::help_navigator::{HelpPage, build_help_view, build_visible_pages};
use crate::events::permission::{is_bot_admin_server, resolve_bot_control};
use crate::types::{PoiseContext, Result};

#[poise::command(
    slash_command,
    name_localized("ja", "ヘルプ"),
    description_localized("ja", "ヘルプを表示します"),
    ephemeral = true
)]
pub async fn help(ctx: PoiseContext<'_>) -> Result<()> {
    let has_bot_control = resolve_bot_control(&ctx).await;
    let is_admin_server = is_bot_admin_server(&ctx).await.unwrap_or(false);
    let guild_id = ctx.guild_id().map(|id| id.get() as i64);
    let locale = get_locale_from_context(&ctx).await;
    let visible_pages = build_visible_pages(has_bot_control, is_admin_server);
    let view = build_help_view(
        &ctx.data().app_state,
        guild_id,
        &locale,
        HelpPage::Index,
        &visible_pages,
    )
    .await;

    ctx.send(
        poise::CreateReply::default()
            .embed(view.embed)
            .components(view.components)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
