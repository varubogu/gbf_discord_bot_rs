//! 募集変更コンポーネントインタラクションの処理。
//!
//! イベント受け取り（ディスパッチ）と、サブモジュール間で共有する定数・
//! ヘルパーのみをここに置く。個々の処理は責務ごとにサブモジュールへ分割する。

use crate::errors::RecruitmentError;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use std::collections::HashMap;

mod apply;
mod apply_responses;
mod date_actions;
mod event_date_label;
mod panel;
mod panel_select_menus;
mod quest_style;

pub use date_actions::set_event_date_draft;
pub use panel::build_panel_content_and_components;

pub(super) const QUEST_NONE_VALUE: &str = "__none_quest__";
pub(super) const STYLE_NONE_VALUE: &str = "__none_style__";

pub(super) const ID_PREFIX_QUEST: &str = "recruit_change_quest";
pub(super) const ID_PREFIX_STYLE: &str = "recruit_change_style";
pub(super) const ID_PREFIX_OPEN_DATE_MODAL: &str = "recruit_change_open_date_modal";
pub(super) const ID_PREFIX_CLEAR_DATE: &str = "recruit_change_clear_date";
pub(super) const ID_PREFIX_APPLY: &str = "recruit_change_apply";

/// メッセージを取得し、失敗時はフォールバック文言を返す共通ヘルパー
pub(super) async fn get_message_or_fallback(
    data: &PoiseData,
    guild_id: Option<u64>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    locale: &str,
    fallback_text: &str,
) -> String {
    data.app_state
        .message_service()
        .get_message(
            data.app_state.guild_db(),
            message_id.as_str(),
            params,
            guild_id.map(|id| id as i64),
            Some(locale),
        )
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}

/// 募集変更関連のコンポーネントインタラクションを処理
pub async fn handle_recruit_change_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with(&format!("{ID_PREFIX_QUEST}:")) {
        quest_style::handle_quest_selection(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_STYLE}:")) {
        quest_style::handle_battle_style_selection(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_OPEN_DATE_MODAL}:")) {
        date_actions::handle_open_date_modal(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_CLEAR_DATE}:")) {
        date_actions::handle_clear_date(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_APPLY}:")) {
        apply::handle_apply_changes(ctx, interaction, data).await
    } else {
        Ok(())
    }
}

/// custom_idから対象チャンネルID・メッセージIDを取り出す
pub(super) fn parse_target_ids(custom_id: &str, prefix: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() != 3 || parts[0] != prefix {
        return Err(AppError::from(RecruitmentError::InvalidCustomId));
    }

    let channel_id = parts[1].parse::<u64>().map_err(|_| {
        AppError::from(RecruitmentError::ParseFailed {
            field: "チャンネルID",
        })
    })?;
    let message_id = parts[2].parse::<u64>().map_err(|_| {
        AppError::from(RecruitmentError::ParseFailed {
            field: "メッセージID",
        })
    })?;

    Ok((channel_id, message_id))
}

#[cfg(test)]
mod tests {
    use super::parse_target_ids;

    #[test]
    fn parse_target_ids_works() {
        let (channel_id, message_id) =
            parse_target_ids("recruit_change_apply:10:20", "recruit_change_apply")
                .expect("custom_id は解析できるべき");
        assert_eq!(channel_id, 10);
        assert_eq!(message_id, 20);
    }

    #[test]
    fn parse_target_ids_rejects_invalid_prefix() {
        let err = parse_target_ids("recruit_change_style:10:20", "recruit_change_apply")
            .expect_err("prefix不一致は失敗するべき");
        assert!(err.user_message().contains("不正なカスタムID"));
    }
}
