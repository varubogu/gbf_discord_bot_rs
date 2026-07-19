//! 募集変更パネルの本文・コンポーネント組み立て。

use crate::events::helpers::resolve_guild_locale;
use crate::facades::recruitment::change_draft::RecruitChangeDraftFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseData, RecruitChangeDraftKey, Result};
use poise::serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton};
use std::collections::HashMap;

use super::{
    ID_PREFIX_APPLY, ID_PREFIX_CLEAR_DATE, ID_PREFIX_OPEN_DATE_MODAL, event_date_label,
    get_message_or_fallback, panel_select_menus,
};

/// パネル表示用の本文とコンポーネントを作成
pub async fn build_panel_content_and_components(
    data: &PoiseData,
    user_id: u64,
    channel_id: u64,
    message_id: u64,
    guild_id: Option<u64>,
) -> Result<(String, Vec<CreateActionRow>)> {
    let locale = resolve_guild_locale(&data.app_state, guild_id.map(|id| id as i64)).await;
    let key = RecruitChangeDraftKey {
        user_id,
        channel_id,
        message_id,
    };

    let draft = RecruitChangeDraftFacade::get(&data.app_state, &key).await;

    let unchanged_quest = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelUnchanged,
        HashMap::new(),
        &locale,
        "変更しない",
    )
    .await;
    let unchanged_style = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelUnchanged,
        HashMap::new(),
        &locale,
        "変更しない",
    )
    .await;
    let apply_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonApply,
        HashMap::new(),
        &locale,
        "適用",
    )
    .await;

    let quest_label = draft
        .quest_name
        .clone()
        .unwrap_or_else(|| unchanged_quest.clone());
    let style_label = draft
        .battle_style_name
        .clone()
        .unwrap_or_else(|| unchanged_style.clone());
    let date_label =
        event_date_label::format_event_date_label(data, guild_id, draft.event_date, &locale).await;

    let mut content_params = HashMap::new();
    content_params.insert("quest_label".to_string(), quest_label.clone());
    content_params.insert("style_label".to_string(), style_label.clone());
    content_params.insert("date_label".to_string(), date_label.clone());
    content_params.insert("apply_label".to_string(), apply_label.clone());
    let content = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelContent,
        content_params,
        &locale,
        &format!(
            "変更内容を選択・入力してください。\n\n\
             現在の入力値\n\
             - クエスト: {quest_label}\n\
             - 攻略方法: {style_label}\n\
             - 出発日時: {date_label}\n\n\
             `{apply_label}`を押すまで反映されません。"
        ),
    )
    .await;

    let quest_select = panel_select_menus::build_quest_select_menu(
        data, guild_id, &locale, channel_id, message_id, &draft,
    )
    .await;
    let style_select = panel_select_menus::build_style_select_menu(
        data, guild_id, &locale, channel_id, message_id, &draft,
    )
    .await;
    let buttons =
        build_action_buttons(data, guild_id, &locale, channel_id, message_id, apply_label).await;

    Ok((
        content,
        vec![
            CreateActionRow::SelectMenu(quest_select),
            CreateActionRow::SelectMenu(style_select),
            CreateActionRow::Buttons(buttons),
        ],
    ))
}

/// 日時操作・適用ボタン群を組み立てる
async fn build_action_buttons(
    data: &PoiseData,
    guild_id: Option<u64>,
    locale: &str,
    channel_id: u64,
    message_id: u64,
    apply_label: String,
) -> Vec<CreateButton> {
    let open_date_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonOpenDate,
        HashMap::new(),
        locale,
        "出発日時を入力",
    )
    .await;
    let clear_date_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonClearDate,
        HashMap::new(),
        locale,
        "日時をクリア",
    )
    .await;

    let open_date_button = CreateButton::new(format!(
        "{ID_PREFIX_OPEN_DATE_MODAL}:{channel_id}:{message_id}"
    ))
    .style(ButtonStyle::Primary)
    .label(open_date_label);

    let clear_date_button =
        CreateButton::new(format!("{ID_PREFIX_CLEAR_DATE}:{channel_id}:{message_id}"))
            .style(ButtonStyle::Secondary)
            .label(clear_date_label);

    let apply_button = CreateButton::new(format!("{ID_PREFIX_APPLY}:{channel_id}:{message_id}"))
        .style(ButtonStyle::Success)
        .label(apply_label);

    vec![open_date_button, clear_date_button, apply_button]
}
