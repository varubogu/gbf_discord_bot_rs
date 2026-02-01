use crate::events::permission::check_bot_control_role;
use crate::facades::schedule::NotificationScheduleFacade;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
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
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

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

    if formatted.is_empty() {
        let embed = CreateEmbed::default()
            .title("📅 スケジュール一覧")
            .description("登録されているスケジュールはありません。\n\n`/schedule_generate` コマンドでスケジュールを生成してください。")
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("📅 スケジュール一覧")
        .description(formatted)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(
            "未来のスケジュールを最大10件まで表示",
        ));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
