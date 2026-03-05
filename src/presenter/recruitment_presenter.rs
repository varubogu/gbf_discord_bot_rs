//! 募集関連プレゼンター
//!
//! 募集表示（Embed、ボタン、セレクトメニュー）の作成を担当する。
//! Service層からUIビルダー依存を除去するために使用する。

use crate::services::guild_environment_service::ElementEmojis;
use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, ButtonContent, ButtonStyleType, EmbedContent, SelectMenuContent,
    SelectMenuOptionContent,
};
use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES, SIMPLE_JOIN_EMOJI};
use rust_i18n::t;

/// 募集表示を担当するPresenter
///
/// 募集関連のEmbed、ボタン、セレクトメニューを生成する。
/// poise/serenity型は使用せず、ドメインモデルを返す。
pub struct RecruitmentPresenter;

fn localized_ja(message_id: MessageTextId) -> String {
    t!(message_id.as_str(), locale = "ja").to_string()
}

fn localized_ja_with_params(message_id: MessageTextId, params: &[(&str, String)]) -> String {
    let mut text = localized_ja(message_id);
    for (key, value) in params {
        text = text.replace(&format!("{{{{{key}}}}}"), value);
    }
    text
}

impl RecruitmentPresenter {
    /// 参加者一覧Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `participants_text` - 参加者一覧テキスト
    /// * `participant_count` - 参加者数（フッター表示用、Noneの場合はフッターなし）
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_participants_embed(
        participants_text: &str,
        participant_count: Option<usize>,
    ) -> EmbedContent {
        let mut embed = EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::RecruitmentPresenterParticipantsTitle,
            ))
            .with_description(participants_text)
            .with_color(0x0099ff);

        if let Some(count) = participant_count {
            let footer = localized_ja_with_params(
                MessageTextId::RecruitmentPresenterParticipantsFooterCount,
                &[("count", count.to_string())],
            );
            embed = embed.with_footer(footer);
        }

        embed
    }

    /// 初期参加者一覧テキストを生成する（ボタン版）
    ///
    /// 全ての枠が「なし」の状態のテキストを返す。
    ///
    /// # Arguments
    ///
    /// * `battle_style_name` - 攻略方法の名前（「6属性」かどうかで分岐）
    /// * `element_emojis` - カスタム属性絵文字
    ///
    /// # Returns
    ///
    /// 初期参加者一覧テキスト
    pub fn create_initial_participants_text(
        battle_style_name: &str,
        element_emojis: &ElementEmojis,
    ) -> String {
        let no_participants = localized_ja(MessageTextId::RecruitmentDisplayNoParticipants);
        let all_elements = localized_ja(MessageTextId::RecruitmentDisplayAllElements);
        let join_label = localized_ja(MessageTextId::RecruitmentPresenterParticipantsJoinLabel);

        if battle_style_name == "6属性" {
            let mut text = String::new();
            let emojis_array = element_emojis.as_array();
            for (emoji, name) in emojis_array.iter().zip(ELEMENT_NAMES.iter()) {
                text.push_str(&format!("{emoji} {name}: {no_participants}\n"));
            }
            text.push_str(&format!(
                "{ALL_ELEMENTS_EMOJI} {all_elements}: {no_participants}\n"
            ));
            text
        } else {
            format!("{SIMPLE_JOIN_EMOJI} {join_label}: {no_participants}\n")
        }
    }

    /// 募集用ボタンを生成する（ボタン版募集用）
    ///
    /// # Arguments
    ///
    /// * `battle_style_name` - 攻略方法の名前（「6属性」かどうかで分岐）
    /// * `element_emojis` - カスタム属性絵文字
    ///
    /// # Returns
    ///
    /// ActionRowContentのVec
    pub fn create_recruitment_buttons(
        battle_style_name: &str,
        element_emojis: &ElementEmojis,
    ) -> Vec<ActionRowContent> {
        if battle_style_name == "6属性" {
            Self::create_six_element_buttons(element_emojis)
        } else {
            Self::create_simple_buttons()
        }
    }

    /// 6属性用ボタンを生成する
    fn create_six_element_buttons(element_emojis: &ElementEmojis) -> Vec<ActionRowContent> {
        let mut element_buttons = Vec::new();
        let emojis_array = element_emojis.as_array();

        for (i, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let index = i + 1;
            let button =
                ButtonContent::new(format!("recruit_join_{index}"), format!("{emoji} {name}"))
                    .with_style(ButtonStyleType::Primary);
            element_buttons.push(button);
        }

        // 全属性可能ボタン
        let all_elements = localized_ja(MessageTextId::RecruitmentDisplayAllElements);
        let all_elements_button = ButtonContent::new(
            "recruit_join_0",
            format!("{ALL_ELEMENTS_EMOJI} {all_elements}"),
        )
        .with_style(ButtonStyleType::Success);

        // 全て取り消しボタン
        let leave_all_button = ButtonContent::new(
            "recruit_leave_all",
            localized_ja(MessageTextId::RecruitmentDisplayLeaveAllButton),
        )
        .with_style(ButtonStyleType::Danger);

        // 行1: 属性1-3
        let row1 = ActionRowContent::buttons(element_buttons[0..3].to_vec());
        // 行2: 属性4-6
        let row2 = ActionRowContent::buttons(element_buttons[3..6].to_vec());
        // 行3: 全属性可能 + 全て取り消し
        let row3 = ActionRowContent::buttons(vec![all_elements_button, leave_all_button]);

        vec![row1, row2, row3]
    }

    /// シンプル参加用ボタンを生成する
    fn create_simple_buttons() -> Vec<ActionRowContent> {
        let join_label = localized_ja(MessageTextId::RecruitmentPresenterParticipantsJoinLabel);
        let join_button =
            ButtonContent::new("recruit_join", format!("{SIMPLE_JOIN_EMOJI} {join_label}"))
                .with_style(ButtonStyleType::Success);

        let leave_all_button = ButtonContent::new(
            "recruit_leave_all",
            localized_ja(MessageTextId::RecruitmentDisplayLeaveAllButton),
        )
        .with_style(ButtonStyleType::Danger);

        let row = ActionRowContent::buttons(vec![join_button, leave_all_button]);
        vec![row]
    }

    /// 属性セレクトメニュー（複数選択可能）を生成する
    ///
    /// # Arguments
    ///
    /// * `element_emojis` - カスタム属性絵文字
    ///
    /// # Returns
    ///
    /// ActionRowContent
    pub fn create_element_select_menu(element_emojis: &ElementEmojis) -> ActionRowContent {
        let emojis_array = element_emojis.as_array();
        let mut options = Vec::new();

        // 属性1-6のオプション
        for (i, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let index = i + 1;
            let option =
                SelectMenuOptionContent::new(format!("{emoji} {name}"), format!("{index}"));
            options.push(option);
        }

        let menu = SelectMenuContent::string_select("recruit_select_elements", options)
            .with_placeholder(localized_ja(
                MessageTextId::RecruitmentPresenterElementSelectPlaceholder,
            ))
            .with_min_values(1)
            .with_max_values(6);

        ActionRowContent::select_menu(menu)
    }

    /// 6属性募集用の全コンポーネント（ボタン + セレクトメニュー）を生成する
    ///
    /// ボタン行の間にセレクトメニューを挿入した完全なレイアウトを返す。
    ///
    /// # Arguments
    ///
    /// * `element_emojis` - カスタム属性絵文字
    ///
    /// # Returns
    ///
    /// ActionRowContentのVec
    pub fn create_six_element_full_components(
        element_emojis: &ElementEmojis,
    ) -> Vec<ActionRowContent> {
        let mut components = Self::create_six_element_buttons(element_emojis);

        // 最後の行（全属性可能＋全て取り消し）を取り出す
        let last_row = components.pop();

        // セレクトメニューを追加
        let select_menu_row = Self::create_element_select_menu(element_emojis);
        components.push(select_menu_row);

        // 最後の行を戻す
        if let Some(row) = last_row {
            components.push(row);
        }

        components
    }

    /// キャンセル確認Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `quest_name` - クエスト名
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_cancel_confirmation_embed(quest_name: &str) -> EmbedContent {
        EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::RecruitmentPresenterCancelConfirmTitle,
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::RecruitmentPresenterCancelConfirmDescription,
                &[("quest_name", quest_name.to_string())],
            ))
            .with_color(0xff0000)
    }

    /// キャンセル確認ボタンを生成する
    ///
    /// # Arguments
    ///
    /// * `recruitment_id` - 募集ID
    ///
    /// # Returns
    ///
    /// ActionRowContent
    pub fn create_cancel_confirmation_buttons(recruitment_id: i64) -> ActionRowContent {
        let confirm_button = ButtonContent::new(
            format!("confirm_cancel_{recruitment_id}"),
            localized_ja(MessageTextId::RecruitmentPresenterCancelConfirmButtonConfirm),
        )
        .with_style(ButtonStyleType::Danger);

        let abort_button = ButtonContent::new(
            format!("abort_cancel_{recruitment_id}"),
            localized_ja(MessageTextId::RecruitmentPresenterCancelConfirmButtonAbort),
        )
        .with_style(ButtonStyleType::Secondary);

        ActionRowContent::buttons(vec![confirm_button, abort_button])
    }

    /// 募集終了Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `quest_name` - クエスト名
    /// * `reason` - 終了理由（"キャンセル", "出発", "解散"等）
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_recruitment_ended_embed(quest_name: &str, reason: &str) -> EmbedContent {
        let cancel_reason = localized_ja(MessageTextId::RecruitmentPresenterEndReasonCancel);
        let departure_reason = localized_ja(MessageTextId::RecruitmentPresenterEndReasonDeparture);
        let dismissal_reason = localized_ja(MessageTextId::RecruitmentPresenterEndReasonDismissal);

        let normalized_reason = if reason == cancel_reason {
            cancel_reason.clone()
        } else if reason == departure_reason {
            departure_reason.clone()
        } else if reason == dismissal_reason {
            dismissal_reason.clone()
        } else {
            reason.to_string()
        };

        let color = if normalized_reason == cancel_reason {
            0xff0000
        } else if normalized_reason == departure_reason {
            0x00ff00
        } else {
            0x808080
        };

        EmbedContent::new()
            .with_title(localized_ja_with_params(
                MessageTextId::RecruitmentPresenterEndTitle,
                &[("reason", normalized_reason.clone())],
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::RecruitmentPresenterEndDescription,
                &[
                    ("quest_name", quest_name.to_string()),
                    ("reason", normalized_reason),
                ],
            ))
            .with_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_element_emojis() -> ElementEmojis {
        ElementEmojis::default_emojis()
    }

    #[test]
    fn test_create_participants_embed() {
        let embed = RecruitmentPresenter::create_participants_embed("🔥 火属性: なし\n", Some(0));

        assert_eq!(embed.title, Some("参加者一覧".to_string()));
        assert!(embed.description.is_some());
        assert_eq!(embed.color, Some(0x0099ff));
    }

    #[test]
    fn test_create_initial_participants_text_six_elements() {
        let emojis = create_test_element_emojis();
        let text = RecruitmentPresenter::create_initial_participants_text("6属性", &emojis);

        assert!(text.contains("火: なし"));
        assert!(text.contains("水: なし"));
        assert!(text.contains("全属性可能: なし"));
    }

    #[test]
    fn test_create_initial_participants_text_simple() {
        let emojis = create_test_element_emojis();
        let text = RecruitmentPresenter::create_initial_participants_text("通常", &emojis);

        assert!(text.contains("参加: なし"));
    }

    #[test]
    fn test_create_recruitment_buttons_six_elements() {
        let emojis = create_test_element_emojis();
        let buttons = RecruitmentPresenter::create_recruitment_buttons("6属性", &emojis);

        // 3行のボタン
        assert_eq!(buttons.len(), 3);
    }

    #[test]
    fn test_create_recruitment_buttons_simple() {
        let emojis = create_test_element_emojis();
        let buttons = RecruitmentPresenter::create_recruitment_buttons("通常", &emojis);

        // 1行のボタン
        assert_eq!(buttons.len(), 1);
    }

    #[test]
    fn test_create_element_select_menu() {
        let emojis = create_test_element_emojis();
        let menu = RecruitmentPresenter::create_element_select_menu(&emojis);

        // セレクトメニューが含まれている
        assert_eq!(menu.components.len(), 1);
    }

    #[test]
    fn test_create_six_element_full_components() {
        let emojis = create_test_element_emojis();
        let components = RecruitmentPresenter::create_six_element_full_components(&emojis);

        // 4行: ボタン2行 + セレクトメニュー1行 + 全属性+取り消しボタン1行
        assert_eq!(components.len(), 4);
    }

    #[test]
    fn test_create_cancel_confirmation_embed() {
        let embed = RecruitmentPresenter::create_cancel_confirmation_embed("天元たる六色の理");

        assert_eq!(embed.title, Some("キャンセル確認".to_string()));
        assert!(
            embed
                .description
                .as_ref()
                .unwrap()
                .contains("天元たる六色の理")
        );
        assert_eq!(embed.color, Some(0xff0000));
    }

    #[test]
    fn test_create_cancel_confirmation_buttons() {
        let buttons = RecruitmentPresenter::create_cancel_confirmation_buttons(123);

        assert_eq!(buttons.components.len(), 2);
    }

    #[test]
    fn test_create_recruitment_ended_embed() {
        let embed = RecruitmentPresenter::create_recruitment_ended_embed("テストクエスト", "出発");

        assert_eq!(embed.title, Some("募集出発".to_string()));
        assert!(embed.description.as_ref().unwrap().contains("出発しました"));
        assert_eq!(embed.color, Some(0x00ff00));
    }
}
