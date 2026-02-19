use crate::events::helpers::{get_message_from_context, get_message_or_fallback_from_context};
use crate::events::permission::check_bot_control_role;
use crate::facades::schedule::ScheduleQueryFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use chrono::Duration;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// 通知統計を表示
///
/// 指定した期間の通知統計を表示します。
#[poise::command(
    slash_command,
    rename = "schedule_stats",
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スケジュール統計"),
    description_localized(
        "ja",
        "指定した期間の通知統計を表示します。（管理者専用サーバーのみ実施可能）"
    )
)]
pub async fn schedule_stats(
    ctx: PoiseContext<'_>,
    #[description = "統計期間（日数、デフォルト: 7日）"]
    #[min = 1]
    #[max = 90]
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
        "通知統計コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = ScheduleQueryFacade::new(Arc::new(app_state.clone()));
    let stats = facade.get_stats(guild_id.get() as i64, days).await?;

    // 表示用にJST変換（UTC+9）
    let from_jst = (stats.from + Duration::hours(9))
        .format("%Y/%m/%d %H:%M")
        .to_string();
    let to_jst = (stats.to + Duration::hours(9))
        .format("%Y/%m/%d %H:%M")
        .to_string();

    let mut header_params = HashMap::new();
    header_params.insert("from".to_string(), from_jst.clone());
    header_params.insert("to".to_string(), to_jst.clone());
    header_params.insert("total_count".to_string(), stats.total_count.to_string());
    let mut description = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandStatsDescriptionHeader,
        header_params,
    )
    .await
    .unwrap_or_else(|_| {
        format!(
            "**期間**: {from_jst} 〜 {to_jst} (JST)\n\n**総通知数**: {} 件\n\n",
            stats.total_count
        )
    });

    if !stats.message_type_counts.is_empty() {
        let message_type_header = get_message_or_fallback_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::ScheduleCommandStatsMessageTypeHeader,
            HashMap::new(),
            "**メッセージタイプ別内訳**:\n",
        )
        .await;
        description.push_str(&message_type_header);

        // 件数の多い順にソート
        let mut counts: Vec<_> = stats.message_type_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for (message_id, count) in counts.iter().take(10) {
            description.push_str(&format!("- `{message_id}`: {count} 件\n"));
        }

        if counts.len() > 10 {
            let mut params = HashMap::new();
            params.insert("count".to_string(), (counts.len() - 10).to_string());
            let other_types = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandStatsOtherTypes,
                params,
            )
            .await
            .unwrap_or_else(|_| format!("\n*...他 {} 種類*\n", counts.len() - 10));
            description.push_str(&other_types);
        }
    }

    let mut title_params = HashMap::new();
    title_params.insert("days".to_string(), days.to_string());
    let title = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandStatsTitleWithDays,
        title_params,
    )
    .await
    .unwrap_or_else(|_| format!("📊 通知統計（過去{days}日間）"));
    let footer = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandStatsFooter,
        HashMap::new(),
        "詳細な統計情報",
    )
    .await;

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(footer));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
