use crate::events::helpers::{get_message_from_context, resolve_guild_locale};
use crate::events::permission::resolve_bot_control;
use crate::facades::recruitment::cancel as CancelFacade;
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::types;
use crate::types::PoiseContext;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use crate::types::domain_interface_result::CanCancelResult;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, Message,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

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

    // events層でpoise型からドメイン型への変換を行う
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

    // 実行者情報を解決（events層でDiscordコンテキストから取得し、ドメイン値として渡す）
    let invoker_user_id = ctx.author().id.get();
    let has_bot_control = resolve_bot_control(&ctx).await;

    // キャンセル可能か確認（権限チェック含む）
    match CancelFacade::can_cancel(
        app_state,
        &gateway,
        guild_id,
        channel_id,
        message_id,
        invoker_user_id,
        has_bot_control,
    )
    .await
    {
        Ok(CanCancelResult::Success) => {
            // キャンセル可能な場合、確認付きでキャンセル処理を実行（events層でUI操作）
            execute_cancel_with_confirmation(ctx, &message).await
        }
        Ok(CanCancelResult::AlreadyCancelled) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelAlreadyCancelled,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelAlreadyCancelled
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Ok(CanCancelResult::MessageDeleted) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelMessageDeleted,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelMessageDeleted
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotRecruitMessage) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelInvalidMessage,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelInvalidMessage
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotFound) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelNotFound,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelNotFound
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Ok(CanCancelResult::EventDatePassed) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelEventDatePassed,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelEventDatePassed
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Ok(CanCancelResult::PermissionDenied) => {
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelPermissionDenied,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelPermissionDenied
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            Ok(())
        }
        Err(e) => {
            // システムエラーを想定
            error!("{:?}", e);
            let msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelError,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelError
                    .as_str()
                    .to_string()
            });

            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            // エラーの種類に関わらずBotは続行
            Ok(())
        }
    }
}

/// キャンセル確認付きで実行（events層でUI操作）
async fn execute_cancel_with_confirmation(
    ctx: PoiseContext<'_>,
    message: &Message,
) -> types::Result<()> {
    let yes_label = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::CommonYes,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| MessageTextId::CommonYes.as_str().to_string());
    let no_label = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::CommonNo,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| MessageTextId::CommonNo.as_str().to_string());
    let confirm_prompt = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentCommandCancelConfirmPrompt,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| {
        MessageTextId::RecruitmentCommandCancelConfirmPrompt
            .as_str()
            .to_string()
    });

    // 確認ボタンを表示
    let confirm_button = CreateButton::new("confirm_cancel")
        .style(ButtonStyle::Danger)
        .label(yes_label);

    let cancel_button = CreateButton::new("deny_cancel")
        .style(ButtonStyle::Secondary)
        .label(no_label);

    let action_row = CreateActionRow::Buttons(vec![confirm_button, cancel_button]);

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(confirm_prompt)
                .components(vec![action_row])
                .ephemeral(true),
        )
        .await?;

    // ユーザーの応答を待機
    let component_interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(move |mci| {
            mci.data.custom_id.starts_with("confirm_cancel")
                || mci.data.custom_id.starts_with("deny_cancel")
        })
        .await;

    match component_interaction {
        Some(interaction) => {
            interaction.defer(&ctx.http()).await?;

            match interaction.data.custom_id.as_str() {
                "confirm_cancel" => {
                    // 「キャンセル中...」に更新
                    let cancelling_message = get_message_from_context(
                        &ctx,
                        ctx.data().app_state.message_service(),
                        MessageTextId::RecruitmentCommandCancellingProgress,
                        HashMap::new(),
                    )
                    .await
                    .unwrap_or_else(|_| {
                        MessageTextId::RecruitmentCommandCancellingProgress
                            .as_str()
                            .to_string()
                    });
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(cancelling_message)
                                .components(vec![]),
                        )
                        .await?;

                    let guild_id = ctx
                        .guild_id()
                        .ok_or_else(|| types::AppError::Business {
                            message: MessageTextId::ErrorsGuildOnly.as_str().to_string(),
                        })?
                        .get();

                    let channel_id = message.channel_id.get();
                    let message_id = message.id.get();

                    // events層でGatewayを作成
                    let app_state = &ctx.data().app_state;
                    let gateway =
                        PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
                    let locale = resolve_guild_locale(app_state, Some(guild_id as i64)).await;

                    // キャンセル実行（facade層）
                    match CancelFacade::execute_cancel(
                        app_state,
                        &gateway,
                        guild_id,
                        channel_id,
                        message_id,
                        Some(locale.as_str()),
                    )
                    .await
                    {
                        Ok(_) => {
                            // 成功時：確認メッセージを更新して完了表示
                            let success_message = get_message_from_context(
                                &ctx,
                                ctx.data().app_state.message_service(),
                                MessageTextId::RecruitmentCommandCancelNotificationNoParticipants,
                                HashMap::new(),
                            )
                            .await
                            .unwrap_or_else(|_| {
                                MessageTextId::RecruitmentCommandCancelNotificationNoParticipants
                                    .as_str()
                                    .to_string()
                            });
                            reply
                                .edit(
                                    ctx,
                                    poise::CreateReply::default()
                                        .content(success_message)
                                        .components(vec![]),
                                )
                                .await?;
                            info!("キャンセル処理完了");
                            Ok(())
                        }
                        Err(e) => {
                            // エラー時：エラーメッセージを表示
                            error!("キャンセル処理エラー: {:?}", e);
                            let error_msg = match &e {
                                types::AppError::Business { message } => message.clone(),
                                _ => MessageTextId::RecruitmentCommandCancelError
                                    .as_str()
                                    .to_string(),
                            };

                            reply
                                .edit(
                                    ctx,
                                    poise::CreateReply::default()
                                        .content(error_msg)
                                        .components(vec![]),
                                )
                                .await?;
                            Err(e)
                        }
                    }
                }
                "deny_cancel" => {
                    // キャンセルを取りやめ
                    let aborted_message = get_message_from_context(
                        &ctx,
                        ctx.data().app_state.message_service(),
                        MessageTextId::RecruitmentCommandCancelAborted,
                        HashMap::new(),
                    )
                    .await
                    .unwrap_or_else(|_| {
                        MessageTextId::RecruitmentCommandCancelAborted
                            .as_str()
                            .to_string()
                    });
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(aborted_message)
                                .components(vec![]),
                        )
                        .await?;
                    Ok(())
                }
                _ => {
                    // 不明な選択
                    let unknown_selection_message = get_message_from_context(
                        &ctx,
                        ctx.data().app_state.message_service(),
                        MessageTextId::RecruitmentCommandCancelUnknownSelection,
                        HashMap::new(),
                    )
                    .await
                    .unwrap_or_else(|_| {
                        MessageTextId::RecruitmentCommandCancelUnknownSelection
                            .as_str()
                            .to_string()
                    });
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(unknown_selection_message)
                                .components(vec![]),
                        )
                        .await?;
                    Ok(())
                }
            }
        }
        None => {
            // タイムアウト
            let timeout_message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandCancelTimeout,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                MessageTextId::RecruitmentCommandCancelTimeout
                    .as_str()
                    .to_string()
            });
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(timeout_message)
                        .components(vec![]),
                )
                .await?;
            Ok(())
        }
    }
}
