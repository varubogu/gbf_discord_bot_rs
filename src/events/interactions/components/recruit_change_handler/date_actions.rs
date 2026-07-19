//! 出発日時モーダルの表示・クリア操作、および下書きへの日時反映。

use crate::events::helpers::resolve_guild_locale;
use crate::facades::recruitment::change_draft::RecruitChangeDraftFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseData, RecruitChangeDraftKey, Result};
use chrono::{DateTime, Utc};
use poise::serenity_prelude::{
    ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateModal, InputTextStyle,
};
use std::collections::HashMap;

use super::{ID_PREFIX_CLEAR_DATE, ID_PREFIX_OPEN_DATE_MODAL};
use super::{build_panel_content_and_components, get_message_or_fallback, parse_target_ids};

/// 出発日時入力モーダルを表示する
pub(super) async fn handle_open_date_modal(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_OPEN_DATE_MODAL)?;

    let custom_id = format!("recruit_change_date_modal:{target_channel_id}:{target_message_id}");

    let guild_id = interaction.guild_id.map(|id| id.get());
    let locale = resolve_guild_locale(&data.app_state, guild_id.map(|id| id as i64)).await;
    let modal_title = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalTitle,
        HashMap::new(),
        &locale,
        "出発日時変更",
    )
    .await;
    let modal_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalEventDateLabel,
        HashMap::new(),
        &locale,
        "出発日時",
    )
    .await;
    let modal_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalEventDatePlaceholder,
        HashMap::new(),
        &locale,
        "例: 12/25 22:30",
    )
    .await;

    let modal =
        CreateModal::new(custom_id, modal_title).components(vec![CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, modal_label, "event_date")
                .placeholder(modal_placeholder)
                .required(true),
        )]);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

/// 出発日時の下書きをクリアする
pub(super) async fn handle_clear_date(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_CLEAR_DATE)?;

    let user_id = interaction.user.id.get();
    set_event_date_draft(data, user_id, target_channel_id, target_message_id, None).await;

    let (content, components) = build_panel_content_and_components(
        data,
        user_id,
        target_channel_id,
        target_message_id,
        interaction.guild_id.map(|id| id.get()),
    )
    .await?;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

/// 日時ドラフトを更新
pub async fn set_event_date_draft(
    data: &PoiseData,
    user_id: u64,
    channel_id: u64,
    message_id: u64,
    event_date: Option<DateTime<Utc>>,
) {
    let key = RecruitChangeDraftKey {
        user_id,
        channel_id,
        message_id,
    };
    RecruitChangeDraftFacade::update(&data.app_state, key, |draft| {
        draft.event_date = event_date;
    })
    .await;
}
