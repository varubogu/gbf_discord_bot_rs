use crate::events::helpers::get_message_or_key_from_context;
use crate::events::permission::check_bot_control_role;
use crate::facades::schedule::NotificationScheduleFacade;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use tracing::info;

/// 登録されているスケジュール一覧を表示
///
/// 今後予定されている通知スケジュールを最大10件表示します。
#[poise::command(
    slash_command,
    rename = "schedule_list",
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スケジュール一覧"),
    description_localized(
        "ja",
        "今後予定されている通知スケジュールを最大10件表示します。（管理者専用サーバーのみ実施可能）"
    )
)]
pub async fn schedule_list(ctx: PoiseContext<'_>) -> Result<()> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            let message = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ErrorsGuildOnly,
                HashMap::new(),
            )
            .await;
            return Err(AppError::Business { message });
        }
    };

    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        "スケジュール一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let facade = NotificationScheduleFacade::new(std::sync::Arc::new(app_state.clone()));
    let formatted = facade
        .get_future_notifications_formatted(guild_id.get() as i64, 10)
        .await?;

    let title = get_message_or_key_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandListTitle,
        HashMap::new(),
    )
    .await;

    if formatted.is_empty() {
        let empty_description = get_message_or_key_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::ScheduleCommandListEmptyDescription,
            HashMap::new(),
        )
        .await;

        let embed = CreateEmbed::default()
            .title(title)
            .description(empty_description)
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let footer = get_message_or_key_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandListFooter,
        HashMap::new(),
    )
    .await;

    let embed = CreateEmbed::default()
        .title(title)
        .description(formatted)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(footer));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
