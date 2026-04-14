use crate::events::helpers::{
    get_locale_from_context, get_message_from_context, get_message_or_fallback_from_context,
};
use crate::events::permission::check_bot_control_role;
use crate::facades::schedule::NotificationScheduleFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use crate::utils::datetime_display::format_datetime_with_weekday;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use tracing::info;

/// 過去の通知履歴を表示
///
/// 指定した日数分の過去の通知履歴を表示します。
#[poise::command(
    slash_command,
    rename = "schedule_history",
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スケジュール履歴"),
    description_localized("ja", "過去の通知履歴を表示します。（管理者専用サーバーのみ実施可能）")
)]
pub async fn schedule_history(
    ctx: PoiseContext<'_>,
    #[min = 1]
    #[max = 30]
    #[name_localized("ja", "表示する日数")]
    #[description = "Number of days to display (default: 7)"]
    #[description_localized("ja", "表示する日数（デフォルト: 7日）")]
    days: Option<i64>,
) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

    let days = days.unwrap_or(7);

    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        days = days,
        "通知履歴コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let facade = NotificationScheduleFacade::new(std::sync::Arc::new(app_state.clone()));
    let locale = get_locale_from_context(&ctx).await;

    let now = chrono::Utc::now();
    let from = now - chrono::Duration::days(days);

    let (formatted, stats) = facade
        .get_notification_history_formatted(guild_id.get() as i64, from, 20, &locale)
        .await?;

    let from_display =
        format_datetime_with_weekday(stats.from, "%Y-%m-%d ({weekday}) %H:%M:%S UTC", &locale);
    let to_display =
        format_datetime_with_weekday(stats.to, "%Y-%m-%d ({weekday}) %H:%M:%S UTC", &locale);

    if formatted.is_empty() {
        let title = get_message_or_fallback_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::ScheduleCommandHistoryTitle,
            HashMap::new(),
            "📜 通知履歴",
        )
        .await;
        let mut params = HashMap::new();
        params.insert("days".to_string(), days.to_string());
        let empty_description = get_message_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::ScheduleCommandHistoryEmptyDescription,
            params,
        )
        .await
        .unwrap_or_else(|_| format!("過去 {days} 日間の通知履歴はありません。"));

        let embed = CreateEmbed::default()
            .title(title)
            .description(empty_description)
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let mut title_params = HashMap::new();
    title_params.insert("days".to_string(), days.to_string());
    let title = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandHistoryTitleWithDays,
        title_params,
    )
    .await
    .unwrap_or_else(|_| format!("📜 通知履歴（過去{days}日間）"));

    let mut footer_params = HashMap::new();
    footer_params.insert("total_count".to_string(), stats.total_count.to_string());
    footer_params.insert("from".to_string(), from_display.clone());
    footer_params.insert("to".to_string(), to_display.clone());
    let footer = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandHistoryFooter,
        footer_params,
    )
    .await
    .unwrap_or_else(|_| {
        format!(
            "合計 {} 件（期間: {} 〜 {}）",
            stats.total_count, from_display, to_display
        )
    });

    let embed = CreateEmbed::default()
        .title(title)
        .description(formatted)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(footer));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
