//! クエスト・攻略方法セレクトメニューの選択処理。

use crate::errors::RecruitmentError;
use crate::facades::recruitment::{
    battle_style_list, change_draft::RecruitChangeDraftFacade, quest_list,
};
use crate::types::{AppError, PoiseData, RecruitChangeDraftKey, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

use super::{ID_PREFIX_QUEST, ID_PREFIX_STYLE, QUEST_NONE_VALUE, STYLE_NONE_VALUE};
use super::{build_panel_content_and_components, parse_target_ids};

/// クエスト選択の変更を下書きへ反映する
pub(super) async fn handle_quest_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_QUEST)?;

    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            values.first().ok_or_else(|| {
                AppError::from(RecruitmentError::NotSelected {
                    field: "クエスト"
                })
            })?
        }
        _ => {
            return Err(AppError::from(RecruitmentError::UnexpectedComponentType));
        }
    };

    let user_id = interaction.user.id.get();
    let key = RecruitChangeDraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    let quest_name = if selected_value == QUEST_NONE_VALUE {
        None
    } else {
        let quest_id: i32 = selected_value.parse().map_err(|_| {
            AppError::from(RecruitmentError::ParseFailed {
                field: "クエストID",
            })
        })?;
        Some(
            quest_list::get_quest_name_by_id(&data.app_state, quest_id)
                .await
                .ok_or_else(|| {
                    AppError::from(RecruitmentError::NotFound {
                        field: "クエスト"
                    })
                })?,
        )
    };
    RecruitChangeDraftFacade::update(&data.app_state, key, |draft| {
        draft.quest_name = quest_name;
    })
    .await;

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

/// 攻略方法選択の変更を下書きへ反映する
pub(super) async fn handle_battle_style_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_STYLE)?;

    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            values.first().ok_or_else(|| {
                AppError::from(RecruitmentError::NotSelected {
                    field: "攻略方法"
                })
            })?
        }
        _ => {
            return Err(AppError::from(RecruitmentError::UnexpectedComponentType));
        }
    };

    let user_id = interaction.user.id.get();
    let key = RecruitChangeDraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    let (battle_style_id, battle_style_name) = if selected_value == STYLE_NONE_VALUE {
        (None, None)
    } else {
        let battle_style_id: i32 = selected_value.parse().map_err(|_| {
            AppError::from(RecruitmentError::ParseFailed {
                field: "攻略方法ID",
            })
        })?;
        let battle_style_name =
            battle_style_list::get_battle_style_name_by_id(&data.app_state, battle_style_id)
                .await
                .unwrap_or_else(|| format!("ID:{battle_style_id}"));
        (Some(battle_style_id), Some(battle_style_name))
    };
    RecruitChangeDraftFacade::update(&data.app_state, key, |draft| {
        draft.battle_style_id = battle_style_id;
        draft.battle_style_name = battle_style_name;
    })
    .await;

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
