//! 適用処理の結果（変更なし・成功・権限エラー・失敗）に応じた応答組み立て。

use crate::facades::recruitment::change_draft::RecruitChangeDraftFacade;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseData, RecruitChangeDraftKey, Result};
use poise::serenity_prelude::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use std::collections::HashMap;
use tracing::{error, info};

use super::{build_panel_content_and_components, get_message_or_fallback};

/// 変更項目が1つも指定されていない場合の応答
#[allow(clippy::too_many_arguments)]
pub(super) async fn respond_no_changes(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    user_id: u64,
    channel_id: u64,
    message_id: u64,
    guild_id: u64,
    locale: &str,
) -> Result<()> {
    let (content, components) =
        build_panel_content_and_components(data, user_id, channel_id, message_id, Some(guild_id))
            .await?;
    let no_changes_message = get_message_or_fallback(
        data,
        Some(guild_id),
        MessageTextId::RecruitmentCommandChangeNoChanges,
        HashMap::new(),
        locale,
        "変更項目を少なくとも1つ指定してください。",
    )
    .await;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("{content}\n\n{no_changes_message}"))
                    .components(components),
            ),
        )
        .await?;
    Ok(())
}

/// 変更適用が成功した場合の応答
#[allow(clippy::too_many_arguments)]
pub(super) async fn respond_apply_success(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    key: &RecruitChangeDraftKey,
    user_id: u64,
    target_guild_id: u64,
    target_channel_id: u64,
    target_message_id: u64,
    locale: &str,
) -> Result<()> {
    RecruitChangeDraftFacade::remove(&data.app_state, key).await;

    let success_message = get_message_or_fallback(
        data,
        Some(target_guild_id),
        MessageTextId::RecruitmentCommandChangeSuccess,
        HashMap::new(),
        locale,
        "募集内容を更新しました。",
    )
    .await;

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(success_message)
                .components(vec![]),
        )
        .await?;

    info!(
        user_id = user_id,
        guild_id = target_guild_id,
        channel_id = target_channel_id,
        message_id = target_message_id,
        "募集内容変更が完了しました"
    );

    Ok(())
}

/// 権限エラーで変更を適用できなかった場合の応答
pub(super) async fn respond_apply_permission_denied(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    target_guild_id: u64,
    locale: &str,
) -> Result<()> {
    let error_msg = get_message_or_fallback(
        data,
        Some(target_guild_id),
        MessageTextId::RecruitmentCommandChangePermissionDenied,
        HashMap::new(),
        locale,
        "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。",
    )
    .await;

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(error_msg)
                .components(vec![]),
        )
        .await?;
    Ok(())
}

/// その他のエラーで変更を適用できなかった場合の応答
#[allow(clippy::too_many_arguments)]
pub(super) async fn respond_apply_error(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    error: AppError,
    user_id: u64,
    target_channel_id: u64,
    target_message_id: u64,
    target_guild_id: u64,
    locale: &str,
) -> Result<()> {
    error!(error = %error, "募集内容変更に失敗しました");

    let (content, components) = build_panel_content_and_components(
        data,
        user_id,
        target_channel_id,
        target_message_id,
        Some(target_guild_id),
    )
    .await?;
    let mut error_params = HashMap::new();
    error_params.insert("error_message".to_string(), error.user_message());
    let error_prefix = get_message_or_fallback(
        data,
        Some(target_guild_id),
        MessageTextId::CommonErrorPrefix,
        error_params,
        locale,
        &format!("エラー: {}", error.user_message()),
    )
    .await;

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(format!("{error_prefix}\n\n{content}"))
                .components(components),
        )
        .await?;
    Ok(())
}
