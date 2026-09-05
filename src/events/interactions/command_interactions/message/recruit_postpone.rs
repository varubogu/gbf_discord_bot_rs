//! メッセージコンテキストメニューから募集の出発時間を遅らせる処理。

use crate::events::helpers::{
    format_event_datetime, get_locale_from_context, get_message_or_key_from_context,
};
use crate::events::permission::resolve_bot_control;
use crate::facades::recruitment::change::{
    RecruitmentChangeOutcome, postpone_recruitment_departure,
};
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::types::discord::MessageData;
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

/// 出発時間を遅らせる分数
const POSTPONE_MINUTES: i64 = 30;

/// メッセージコンテキストメニューから募集の出発時間を30分遅らせる
///
/// 「募集内容変更」と同じ更新処理を利用するが、変更対象は出発日時のみで、
/// 実行者からの追加入力は不要とする。
#[poise::command(context_menu_command = "募集を30分遅らせる")]
pub async fn recruit_postpone_context_menu(ctx: PoiseContext<'_>, message: Message) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let invoker_user_id = ctx.author().id.get();
    let has_bot_control = resolve_bot_control(&ctx).await;

    let guild_id = ctx
        .guild_id()
        .map(|id| id.get())
        .or_else(|| message.guild_id.map(|id| id.get()));

    let Some(guild_id) = guild_id else {
        return respond(ctx, MessageTextId::ErrorsGuildOnly, HashMap::new()).await;
    };

    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
    let message_data = MessageData::from(message);

    let result = postpone_recruitment_departure(
        &ctx.data().app_state,
        &gateway,
        guild_id,
        &message_data,
        POSTPONE_MINUTES,
        invoker_user_id,
        has_bot_control,
    )
    .await;

    match result {
        Ok(RecruitmentChangeOutcome::Applied { event_date }) => {
            let locale = get_locale_from_context(&ctx).await;
            let mut params = HashMap::new();
            params.insert("minutes".to_string(), POSTPONE_MINUTES.to_string());
            params.insert(
                "event_date".to_string(),
                format_event_datetime(&ctx.data().app_state, Some(guild_id), event_date, &locale)
                    .await,
            );
            respond(
                ctx,
                MessageTextId::RecruitmentCommandPostponeSuccess,
                params,
            )
            .await
        }
        Ok(RecruitmentChangeOutcome::EventDatePassed) => {
            respond(
                ctx,
                MessageTextId::RecruitmentCommandPostponeEventDatePassed,
                HashMap::new(),
            )
            .await
        }
        // 権限エラーはBusinessエラーとして返るため、専用の文言で通知する
        Err(AppError::Business { .. }) => {
            respond(
                ctx,
                MessageTextId::RecruitmentCommandChangePermissionDenied,
                HashMap::new(),
            )
            .await
        }
        Err(error) => {
            error!(?error, "募集の出発時間を遅らせる処理に失敗しました");
            let mut params = HashMap::new();
            params.insert("error_message".to_string(), error.user_message());
            respond(ctx, MessageTextId::CommonErrorPrefix, params).await
        }
    }
}

/// 実行結果をエフェメラルメッセージで通知する
async fn respond(
    ctx: PoiseContext<'_>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> Result<()> {
    let content = get_message_or_key_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        message_id,
        params,
    )
    .await;

    ctx.send(
        poise::CreateReply::default()
            .content(content)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
