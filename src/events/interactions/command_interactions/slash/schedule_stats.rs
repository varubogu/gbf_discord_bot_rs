use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::NotificationRepository;
use crate::types::{PoiseContext, Result};
use chrono::{Duration, Utc};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use sea_orm::TransactionTrait;
use tracing::{error, info};

/// 通知統計を表示
///
/// 指定した期間の通知統計を表示します。
#[poise::command(
    slash_command,
    rename = "schedule_stats",
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn schedule_stats(
    ctx: PoiseContext<'_>,
    #[description = "統計期間（日数、デフォルト: 7日）"]
    #[min = 1]
    #[max = 90]
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
        "通知統計コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let notification_repo = NotificationRepository::new();

    let now = Utc::now();
    let from = now - Duration::days(days);

    // 統計を取得
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

            let total_count = notifications.len();

            // メッセージタイプ別の集計
            let mut message_type_counts = std::collections::HashMap::new();
            for notification in &notifications {
                *message_type_counts
                    .entry(notification.message_text_id.clone())
                    .or_insert(0) += 1;
            }

            let stats = crate::services::schedule::NotificationStats {
                total_count,
                message_type_counts,
            };
            // 表示用にJST変換（UTC+9）
            let mut description = format!(
                "**期間**: {} 〜 {} (JST)\n\n**総通知数**: {} 件\n\n",
                (from + Duration::hours(9)).format("%Y/%m/%d %H:%M"),
                (now + Duration::hours(9)).format("%Y/%m/%d %H:%M"),
                stats.total_count
            );

            if !stats.message_type_counts.is_empty() {
                description.push_str("**メッセージタイプ別内訳**:\n");

                // 件数の多い順にソート
                let mut counts: Vec<_> = stats.message_type_counts.iter().collect();
                counts.sort_by(|a, b| b.1.cmp(a.1));

                for (message_id, count) in counts.iter().take(10) {
                    description.push_str(&format!("- `{}`: {} 件\n", message_id, count));
                }

                if counts.len() > 10 {
                    description.push_str(&format!("\n*...他 {} 種類*\n", counts.len() - 10));
                }
            }

            let embed = CreateEmbed::default()
                .title(format!("📊 通知統計（過去{}日間）", days))
                .description(description)
                .color(0x00aaff)
                .footer(CreateEmbedFooter::new("詳細な統計情報"));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "通知統計の取得に失敗しました");

            let embed = CreateEmbed::default()
                .title("❌ エラー")
                .description(format!(
                    "通知統計の取得中にエラーが発生しました。\n```\n{}\n```",
                    e
                ))
                .color(0xff0000);

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
