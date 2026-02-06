use crate::events::permission::check_bot_control_role;
use crate::facades::schedule::ScheduleQueryFacade;
use crate::types::{PoiseContext, Result};
use chrono::Duration;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
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
    let mut description = format!(
        "**期間**: {} 〜 {} (JST)\n\n**総通知数**: {} 件\n\n",
        (stats.from + Duration::hours(9)).format("%Y/%m/%d %H:%M"),
        (stats.to + Duration::hours(9)).format("%Y/%m/%d %H:%M"),
        stats.total_count
    );

    if !stats.message_type_counts.is_empty() {
        description.push_str("**メッセージタイプ別内訳**:\n");

        // 件数の多い順にソート
        let mut counts: Vec<_> = stats.message_type_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for (message_id, count) in counts.iter().take(10) {
            description.push_str(&format!("- `{message_id}`: {count} 件\n"));
        }

        if counts.len() > 10 {
            description.push_str(&format!("\n*...他 {} 種類*\n", counts.len() - 10));
        }
    }

    let embed = CreateEmbed::default()
        .title(format!("📊 通知統計（過去{days}日間）"))
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new("詳細な統計情報"));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
