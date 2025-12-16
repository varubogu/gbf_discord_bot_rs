use crate::facades::schedule::NotificationScheduleFacade;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
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

    let now = chrono::Utc::now();
    let from = now - chrono::Duration::days(days);

    let (formatted, stats) = facade
        .get_notification_history_formatted(guild_id.get() as i64, from, 20)
        .await?;

    if formatted.is_empty() {
        let embed = CreateEmbed::default()
            .title("📜 通知履歴")
            .description(format!("過去 {} 日間の通知履歴はありません。", days))
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title(format!("📜 通知履歴（過去{}日間）", days))
        .description(formatted)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(format!(
            "合計 {} 件（期間: {} 〜 {}）",
            stats.total_count, stats.from, stats.to
        )));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
