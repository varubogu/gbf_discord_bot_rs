use crate::events::helpers::{get_message_from_context, get_message_or_fallback_from_context};
use crate::events::interactions::command_interactions::slash::schedule::schedule_task_type::ScheduleTaskTypeChoice;
use crate::events::permission::check_bot_admin_server;
use crate::facades::scheduler::SchedulerFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
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
    let loading_message = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ScheduleCommandGlobalGenerateLoading,
        HashMap::new(),
        "全体スケジュールを生成しています...",
    )
    .await;
    ctx.say(loading_message).await?;

    let app_state = Arc::new(ctx.data().app_state.clone());
    let scheduler_facade = SchedulerFacade::new(app_state);

    // スケジュール生成を実行
    match scheduler_facade
        .generate_schedules_global(task_type.map(Into::into))
        .await
    {
        Ok(_) => {
            info!("全体スケジュール生成が完了しました");

            let title = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateSuccessTitle,
                HashMap::new(),
                "✅ 全体スケジュール生成完了",
            )
            .await;
            let description = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateSuccessDescription,
                HashMap::new(),
                "全guildのイベントスケジュールから通知スケジュールを生成しました。",
            )
            .await;
            let field_name = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateSuccessFieldName,
                HashMap::new(),
                "処理内容",
            )
            .await;
            let field_value = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateSuccessFieldValue,
                HashMap::new(),
                "- 対象の既存スケジュールを削除\n- イベントスケジュールを読み込み\n- 通知スケジュールを計算・保存",
            )
            .await;
            let footer = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateSuccessFooter,
                HashMap::new(),
                "10秒間隔で自動的に通知が送信されます",
            )
            .await;

            let embed = CreateEmbed::default()
                .title(title)
                .description(description)
                .color(0x00ff00)
                .field(field_name, field_value, false)
                .footer(CreateEmbedFooter::new(footer));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            error!(error = %e, "全体スケジュール生成に失敗しました");

            let title = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateErrorTitle,
                HashMap::new(),
                "❌ 全体スケジュール生成エラー",
            )
            .await;
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), e.to_string());
            let description = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateErrorDescription,
                params,
            )
            .await
            .unwrap_or_else(|_| {
                format!("全体スケジュールの生成中にエラーが発生しました。\n```\n{e}\n```")
            });
            let footer = get_message_or_fallback_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ScheduleCommandGlobalGenerateErrorFooter,
                HashMap::new(),
                "詳細はログを確認してください",
            )
            .await;

            let embed = CreateEmbed::default()
                .title(title)
                .description(description)
                .color(0xff0000)
                .footer(CreateEmbedFooter::new(footer));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
