use crate::events::interactions::command_interactions::slash::schedule::schedule_task_type::ScheduleTaskTypeChoice;
use crate::events::permission::check_bot_admin_server;
use crate::facades::scheduler::SchedulerFacade;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::sync::Arc;
use tracing::{error, info};

/// 管理サーバー向けスケジュールを生成・更新
///
/// 全guildのイベントスケジュールと詳細から通知スケジュールを再計算してDBに保存します。
#[poise::command(
    slash_command,
    rename = "schedule_global_generate",
    check = "check_bot_admin_server",
    ephemeral = true,
    name_localized("ja", "全体スケジュール生成"),
    description_localized(
        "ja",
        "全guildのスケジュールを再計算してDBに保存します。（管理サーバー専用）"
    )
)]
pub async fn schedule_global_generate(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "タスク種別")]
    #[description = "Task type"]
    #[description_localized("ja", "再生成対象のタスク種別（未指定時は全て）")]
    task_type: Option<ScheduleTaskTypeChoice>,
) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer_ephemeral().await?;

    info!(
        guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0),
        user_id = ctx.author().id.get(),
        task_type = ?task_type,
        "全体スケジュール生成コマンドが実行されました"
    );

    // 処理中メッセージを送信
    ctx.say("全体スケジュールを生成しています...").await?;

    let app_state = Arc::new(ctx.data().app_state.clone());
    let scheduler_facade = SchedulerFacade::new(app_state);

    // スケジュール生成を実行
    match scheduler_facade
        .generate_schedules_global(task_type.map(Into::into))
        .await
    {
        Ok(_) => {
            info!("全体スケジュール生成が完了しました");

            let embed = CreateEmbed::default()
                .title("✅ 全体スケジュール生成完了")
                .description("全guildのイベントスケジュールから通知スケジュールを生成しました。")
                .color(0x00ff00)
                .field(
                    "処理内容",
                    "- 対象の既存スケジュールを削除\n- イベントスケジュールを読み込み\n- 通知スケジュールを計算・保存",
                    false,
                )
                .footer(CreateEmbedFooter::new("10秒間隔で自動的に通知が送信されます"));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            error!(error = %e, "全体スケジュール生成に失敗しました");

            let embed = CreateEmbed::default()
                .title("❌ 全体スケジュール生成エラー")
                .description(format!(
                    "全体スケジュールの生成中にエラーが発生しました。\n```\n{e}\n```"
                ))
                .color(0xff0000)
                .footer(CreateEmbedFooter::new("詳細はログを確認してください"));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
