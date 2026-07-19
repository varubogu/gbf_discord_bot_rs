//! パネル用セレクトメニュー（クエスト・攻略方法）の組み立て。

use crate::facades::recruitment::{battle_style_list, quest_list};
use crate::services::message::MessageTextId;
use crate::types::{PoiseData, RecruitChangeDraft};
use poise::serenity_prelude::{CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};
use std::collections::HashMap;

use super::get_message_or_fallback;
use super::{ID_PREFIX_QUEST, ID_PREFIX_STYLE, QUEST_NONE_VALUE, STYLE_NONE_VALUE};

/// クエスト選択メニューを組み立てる
pub(super) async fn build_quest_select_menu(
    data: &PoiseData,
    guild_id: Option<u64>,
    locale: &str,
    channel_id: u64,
    message_id: u64,
    draft: &RecruitChangeDraft,
) -> CreateSelectMenu {
    let quest_pairs = quest_list::list_quests_for_select(&data.app_state).await;

    let option_quest_unchanged = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeOptionQuestUnchanged,
        HashMap::new(),
        locale,
        "クエスト：変更しない",
    )
    .await;
    let mut quest_options = vec![
        CreateSelectMenuOption::new(option_quest_unchanged, QUEST_NONE_VALUE)
            .default_selection(draft.quest_name.is_none()),
    ];
    quest_options.extend(quest_pairs.into_iter().take(24).map(|(name, id)| {
        let is_selected = draft
            .quest_name
            .as_ref()
            .map(|n| n == &name)
            .unwrap_or(false);
        CreateSelectMenuOption::new(name, id.to_string()).default_selection(is_selected)
    }));

    let quest_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePlaceholderQuest,
        HashMap::new(),
        locale,
        "クエストを選択",
    )
    .await;

    CreateSelectMenu::new(
        format!("{ID_PREFIX_QUEST}:{channel_id}:{message_id}"),
        CreateSelectMenuKind::String {
            options: quest_options,
        },
    )
    .placeholder(quest_placeholder)
}

/// 攻略方法選択メニューを組み立てる
pub(super) async fn build_style_select_menu(
    data: &PoiseData,
    guild_id: Option<u64>,
    locale: &str,
    channel_id: u64,
    message_id: u64,
    draft: &RecruitChangeDraft,
) -> CreateSelectMenu {
    let style_pairs = battle_style_list::list_battle_styles_for_select(&data.app_state).await;
    let option_style_unchanged = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeOptionStyleUnchanged,
        HashMap::new(),
        locale,
        "攻略方法：変更しない",
    )
    .await;
    let mut style_options = vec![
        CreateSelectMenuOption::new(option_style_unchanged, STYLE_NONE_VALUE)
            .default_selection(draft.battle_style_id.is_none()),
    ];
    style_options.extend(style_pairs.into_iter().take(24).map(|(name, id)| {
        let is_selected = draft.battle_style_id.map(|s| s == id).unwrap_or(false);
        CreateSelectMenuOption::new(name, id.to_string()).default_selection(is_selected)
    }));

    let style_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePlaceholderStyle,
        HashMap::new(),
        locale,
        "攻略方法を選択",
    )
    .await;

    CreateSelectMenu::new(
        format!("{ID_PREFIX_STYLE}:{channel_id}:{message_id}"),
        CreateSelectMenuKind::String {
            options: style_options,
        },
    )
    .placeholder(style_placeholder)
}
