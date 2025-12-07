use crate::facades::scheduler::SchedulerFacade;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::sync::Arc;
use tracing::{error, info};

/// スケジュールを生成・更新
///
/// イベントスケジュールと詳細から通知スケジュールを再計算してDBに保存します。
#[poise::command(
    slash_command,
    rename = "schedule_generate",
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn schedule_generate(ctx: PoiseContext<'_>) -> Result<()> {
    info!(
        guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0),
        user_id = ctx.author().id.get(),
        "スケジュール生成コマンドが実行されました"
    );

    // 処理中メッセージを送信
    ctx.send(
        poise::CreateReply::default()
            .content("スケジュールを生成しています...")
            .ephemeral(true),
    )
    .await?;

    let app_state = Arc::new(ctx.data().app_state.clone());
    let scheduler_facade = SchedulerFacade::new(app_state);

    // スケジュール生成を実行
    match scheduler_facade.generate_schedules().await {
        Ok(_) => {
            info!("スケジュール生成が完了しました");

            let embed = CreateEmbed::default()
                .title("✅ スケジュール生成完了")
                .description("イベントスケジュールから通知スケジュールを生成しました。")
                .color(0x00ff00)
                .field(
                    "処理内容",
                    "- 既存のスケジュールを削除\n- イベントスケジュールを読み込み\n- 通知スケジュールを計算・保存",
                    false,
                )
                .footer(CreateEmbedFooter::new("10秒間隔で自動的に通知が送信されます"));

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .ephemeral(true),
            )
                .await?;
        }
        Err(e) => {
            error!(error = %e, "スケジュール生成に失敗しました");

            let embed = CreateEmbed::default()
                .title("❌ スケジュール生成エラー")
                .description(format!("スケジュールの生成中にエラーが発生しました。\n```\n{}\n```", e))
                .color(0xff0000)
                .footer(CreateEmbedFooter::new("詳細はログを確認してください"));

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
