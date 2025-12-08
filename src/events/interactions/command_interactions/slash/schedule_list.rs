use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::NotificationRepository;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use sea_orm::TransactionTrait;
use tracing::{error, info};

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
    description_localized("ja", "今後予定されている通知スケジュールを最大10件表示します。（管理者専用サーバーのみ実施可能）"),
)]
pub async fn schedule_list(ctx: PoiseContext<'_>) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        }
    })?;

    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        "スケジュール一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let notification_repo = NotificationRepository::new();

    // このギルドの通知を取得
    match notification_repo
        .find_by_guild_id_with_txn(&txn, guild_id.get() as i64)
        .await
    {
        Ok(notifications) => {
            txn.commit().await?;
            if notifications.is_empty() {
                let embed = CreateEmbed::default()
                    .title("📅 スケジュール一覧")
                    .description("登録されているスケジュールはありません。\n\n`/schedule_generate` コマンドでスケジュールを生成してください。")
                    .color(0xffaa00);

                ctx.send(
                    poise::CreateReply::default()
                        .embed(embed)
                        .ephemeral(true),
                )
                    .await?;
                return Ok(());
            }

            let now = Utc::now();

            // 未来のスケジュールのみをフィルタ・ソート
            let mut future_notifications: Vec<_> = notifications
                .into_iter()
                .filter(|n| n.schedule_datetime > now)
                .collect();

            future_notifications.sort_by_key(|n| n.schedule_datetime);

            // 最大10件に制限
            let display_count = future_notifications.len().min(10);
            let total_count = future_notifications.len();

            let mut description = String::new();
            for (i, notification) in future_notifications.iter().take(display_count).enumerate() {
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
                    "\n*...他 {} 件のスケジュールがあります*",
                    total_count - display_count
                ));
            }

            let embed = CreateEmbed::default()
                .title("📅 スケジュール一覧")
                .description(description)
                .color(0x00aaff)
                .footer(CreateEmbedFooter::new(format!("合計 {} 件の予定されたスケジュール", total_count)));

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .ephemeral(true),
            )
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "スケジュール一覧の取得に失敗しました");

            let embed = CreateEmbed::default()
                .title("❌ エラー")
                .description(format!(
                    "スケジュール一覧の取得中にエラーが発生しました。\n```\n{}\n```",
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
