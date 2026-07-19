//! 募集変更内容の適用処理。

use crate::errors::RecruitmentError;
use crate::events::helpers::resolve_guild_locale;
use crate::events::permission::resolve_bot_control_for_interaction;
use crate::facades::recruitment::change_draft::RecruitChangeDraftFacade;
use crate::gateway::PoiseDiscordGateway;
use crate::types::discord::MessageData;
use crate::types::{AppError, PoiseData, RecruitChangeDraftKey, Result};
use poise::serenity_prelude::{ChannelId, ComponentInteraction, Context, Message};
use std::sync::Arc;
use tracing::{error, warn};

use super::ID_PREFIX_APPLY;
use super::apply_responses::{
    respond_apply_error, respond_apply_permission_denied, respond_apply_success, respond_no_changes,
};
use super::parse_target_ids;

/// 募集変更パネルの入力内容を確定・反映する
pub(super) async fn handle_apply_changes(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_APPLY)?;

    let user_id = interaction.user.id.get();
    let interaction_guild_id = interaction
        .guild_id
        .ok_or_else(|| {
            AppError::from(RecruitmentError::MissingInput {
                field: "ギルドID"
            })
        })?
        .get();
    let locale = resolve_guild_locale(&data.app_state, Some(interaction_guild_id as i64)).await;

    let key = RecruitChangeDraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    let draft = RecruitChangeDraftFacade::get(&data.app_state, &key).await;

    if draft.quest_name.is_none() && draft.battle_style_id.is_none() && draft.event_date.is_none() {
        return respond_no_changes(
            ctx,
            interaction,
            data,
            user_id,
            target_channel_id,
            target_message_id,
            interaction_guild_id,
            &locale,
        )
        .await;
    }

    interaction.defer(&ctx.http).await?;

    let (target_message, target_guild_id) = fetch_target_message(
        ctx,
        target_channel_id,
        target_message_id,
        interaction_guild_id,
    )
    .await?;

    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));
    let message_data = MessageData::from(target_message);

    // 実行者情報を解決（events層でDiscordコンテキストから取得し、ドメイン値として渡す）
    let has_bot_control = resolve_bot_control_for_interaction(ctx, interaction).await;

    let result = crate::facades::recruitment::change::change_recruitment_information_internal(
        &data.app_state,
        &gateway,
        target_guild_id,
        &message_data,
        crate::facades::recruitment::change::RecruitmentChangeContent {
            quest: draft.quest_name,
            event_date: draft.event_date,
            battle_style_id: draft.battle_style_id,
        },
        user_id,
        has_bot_control,
    )
    .await;

    match result {
        Ok(_) => {
            respond_apply_success(
                ctx,
                interaction,
                data,
                &key,
                user_id,
                target_guild_id,
                target_channel_id,
                target_message_id,
                &locale,
            )
            .await
        }
        Err(AppError::Business { .. }) => {
            respond_apply_permission_denied(ctx, interaction, data, target_guild_id, &locale).await
        }
        Err(e) => {
            respond_apply_error(
                ctx,
                interaction,
                data,
                e,
                user_id,
                target_channel_id,
                target_message_id,
                target_guild_id,
                &locale,
            )
            .await
        }
    }
}

/// 変更対象メッセージを取得し、ギルドIDの不一致があれば警告ログを出す
async fn fetch_target_message(
    ctx: &Context,
    target_channel_id: u64,
    target_message_id: u64,
    interaction_guild_id: u64,
) -> Result<(Message, u64)> {
    let target_message = ChannelId::new(target_channel_id)
        .message(&ctx.http, target_message_id)
        .await
        .map_err(|e| {
            error!(error = %e, channel_id = target_channel_id, message_id = target_message_id, "メッセージの取得に失敗しました");
            AppError::from(RecruitmentError::NotFound {
                field: "対象メッセージ",
            })
        })?;

    let target_guild_id = target_message
        .guild_id
        .map(|id| id.get())
        .unwrap_or(interaction_guild_id);

    if target_guild_id != interaction_guild_id {
        warn!(
            interaction_guild_id = interaction_guild_id,
            target_guild_id = target_guild_id,
            channel_id = target_channel_id,
            message_id = target_message_id,
            "募集変更のギルドIDが一致しません"
        );
    }

    Ok((target_message, target_guild_id))
}
