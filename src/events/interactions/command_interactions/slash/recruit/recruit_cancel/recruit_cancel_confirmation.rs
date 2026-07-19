use crate::events::helpers::{get_message_from_context, resolve_guild_locale};
use crate::facades::recruitment::cancel as CancelFacade;
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::types;
use crate::types::PoiseContext;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, Message,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

async fn localized(ctx: PoiseContext<'_>, message_id: MessageTextId) -> String {
    get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        message_id,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| message_id.as_str().to_string())
}

/// キャンセル確認UIを表示し、確認された場合にFacadeへ実行を委譲する。
pub(super) async fn execute_cancel_with_confirmation(
    ctx: PoiseContext<'_>,
    message: &Message,
) -> types::Result<()> {
    let action_row = CreateActionRow::Buttons(vec![
        CreateButton::new("confirm_cancel")
            .style(ButtonStyle::Danger)
            .label(localized(ctx, MessageTextId::CommonYes).await),
        CreateButton::new("deny_cancel")
            .style(ButtonStyle::Secondary)
            .label(localized(ctx, MessageTextId::CommonNo).await),
    ]);
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(localized(ctx, MessageTextId::RecruitmentCommandCancelConfirmPrompt).await)
                .components(vec![action_row])
                .ephemeral(true),
        )
        .await?;

    let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(|interaction| {
            interaction.data.custom_id == "confirm_cancel"
                || interaction.data.custom_id == "deny_cancel"
        })
        .await;

    match interaction {
        Some(interaction) if interaction.data.custom_id == "confirm_cancel" => {
            interaction.defer(&ctx.http()).await?;
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(
                            localized(ctx, MessageTextId::RecruitmentCommandCancellingProgress)
                                .await,
                        )
                        .components(vec![]),
                )
                .await?;

            let guild_id = ctx
                .guild_id()
                .ok_or_else(|| types::AppError::Business {
                    message: MessageTextId::ErrorsGuildOnly.as_str().to_string(),
                })?
                .get();
            let app_state = &ctx.data().app_state;
            let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
            let locale = resolve_guild_locale(app_state, Some(guild_id as i64)).await;
            match CancelFacade::execute_cancel(
                app_state,
                &gateway,
                guild_id,
                message.channel_id.get(),
                message.id.get(),
                Some(locale.as_str()),
            )
            .await
            {
                Ok(()) => {
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(
                                    localized(
                                        ctx,
                                        MessageTextId::RecruitmentCommandCancelNotificationNoParticipants,
                                    )
                                    .await,
                                )
                                .components(vec![]),
                        )
                        .await?;
                    info!("キャンセル処理完了");
                    Ok(())
                }
                Err(error) => {
                    error!(?error, "募集キャンセル処理に失敗しました");
                    let text = match &error {
                        types::AppError::Business { message } => message.clone(),
                        _ => localized(ctx, MessageTextId::RecruitmentCommandCancelError).await,
                    };
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(text)
                                .components(vec![]),
                        )
                        .await?;
                    Err(error)
                }
            }
        }
        Some(interaction) if interaction.data.custom_id == "deny_cancel" => {
            interaction.defer(&ctx.http()).await?;
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(
                            localized(ctx, MessageTextId::RecruitmentCommandCancelAborted).await,
                        )
                        .components(vec![]),
                )
                .await?;
            Ok(())
        }
        Some(interaction) => {
            interaction.defer(&ctx.http()).await?;
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(
                            localized(ctx, MessageTextId::RecruitmentCommandCancelUnknownSelection)
                                .await,
                        )
                        .components(vec![]),
                )
                .await?;
            Ok(())
        }
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(
                            localized(ctx, MessageTextId::RecruitmentCommandCancelTimeout).await,
                        )
                        .components(vec![]),
                )
                .await?;
            Ok(())
        }
    }
}
