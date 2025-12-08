use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::NotificationRepository;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use sea_orm::TransactionTrait;
use tracing::{error, info};

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
    description_localized("ja", "過去の通知履歴を表示します。（管理者専用サーバーのみ実施可能）"),
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
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        }
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
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let notification_repo = NotificationRepository::new();

    // 過去の通知を取得
    let now = chrono::Utc::now();
    let from = now - chrono::Duration::days(days);

    match notification_repo
        .find_by_datetime_range_with_txn(&txn, from, now)
        .await
    {
        Ok(all_notifications) => {
            txn.commit().await?;

            // ギルドでフィルタ
            let notifications: Vec<_> = all_notifications
                .into_iter()
                .filter(|n| n.guild_id == guild_id.get() as i64)
                .collect();

            if notifications.is_empty() {
                let embed = CreateEmbed::default()
                    .title("📜 通知履歴")
                    .description(format!(
                        "過去 {} 日間の通知履歴はありません。",
                        days
                    ))
                    .color(0xffaa00);

                ctx.send(
                    poise::CreateReply::default()
                        .embed(embed)
                        .ephemeral(true),
                )
                    .await?;
                return Ok(());
            }

            // 日時の降順でソート（新しい順）
            let mut sorted_notifications = notifications;
            sorted_notifications.sort_by(|a, b| b.schedule_datetime.cmp(&a.schedule_datetime));

            // 最大20件に制限
            let display_count = sorted_notifications.len().min(20);
            let total_count = sorted_notifications.len();

            let mut description = String::new();
            for (i, notification) in sorted_notifications.iter().take(display_count).enumerate()
            {
                // 表示用にJST変換（UTC+9）
                let datetime_jst = notification.schedule_datetime + chrono::Duration::hours(9);
                description.push_str(&format!(
                    "{}. **{}** (JST)\n   メッセージID: `{}`\n   チャンネル: <#{}>\n\n",
                    i + 1,
                    datetime_jst.format("%Y/%m/%d %H:%M:%S"),
                    notification.message_text_id,
                    notification.channel_id
                ));
            }

            if total_count > display_count {
                description.push_str(&format!(
                    "\n*...他 {} 件の履歴があります*",
                    total_count - display_count
                ));
            }

            let embed = CreateEmbed::default()
                .title(format!("📜 通知履歴（過去{}日間）", days))
                .description(description)
                .color(0x00aaff)
                .footer(CreateEmbedFooter::new(format!("合計 {} 件の通知", total_count)));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "通知履歴の取得に失敗しました");

            let embed = CreateEmbed::default()
                .title("❌ エラー")
                .description(format!(
                    "通知履歴の取得中にエラーが発生しました。\n```\n{}\n```",
                    e
                ))
                .color(0xff0000);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .ephemeral(true),
            )
                .await?;
        }
    }

    Ok(())
}
