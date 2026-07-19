mod recruit_cancel_confirmation;

use crate::events::helpers::get_message_from_context;
use crate::events::permission::resolve_bot_control;
use crate::facades::recruitment::cancel as CancelFacade;
use crate::gateway::PoiseDiscordGateway;
use crate::presenter::RecruitmentPresenter;
use crate::services::message::MessageTextId;
use crate::types;
use crate::types::PoiseContext;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use poise::serenity_prelude::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

#[poise::command(
    context_menu_command = "募集キャンセル",
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル"),
    ephemeral = true
)]
pub async fn recruit_cancel(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "募集メッセージ")]
    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> types::Result<()> {
    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
    let guild_id = DiscordGuildId::new(
        ctx.guild_id()
            .map(|id| id.get())
            .or_else(|| message.guild_id.map(|id| id.get()))
            .ok_or_else(|| types::AppError::Business {
                message: MessageTextId::ErrorsGuildOnly.as_str().to_string(),
            })?,
    );
    let channel_id = DiscordChannelId::new(message.channel_id.get());
    let message_id = DiscordMessageId::new(message.id.get());

    let result = CancelFacade::can_cancel(
        app_state,
        &gateway,
        guild_id,
        channel_id,
        message_id,
        ctx.author().id.get(),
        resolve_bot_control(&ctx).await,
    )
    .await;

    match result {
        Ok(result) => {
            let Some(message_id) = RecruitmentPresenter::can_cancel_result_message_id(result)
            else {
                return recruit_cancel_confirmation::execute_cancel_with_confirmation(
                    ctx, &message,
                )
                .await;
            };
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                message_id,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| message_id.as_str().to_string());
            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            error!(?error, "募集キャンセル可否の確認に失敗しました");
            let message_id = MessageTextId::RecruitmentCommandCancelError;
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                message_id,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| message_id.as_str().to_string());
            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
    }
}
