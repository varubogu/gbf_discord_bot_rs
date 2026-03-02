//! 自動募集UIプレゼンター
//!
//! 自動募集関連のUI（クエスト選択、属性選択等）の作成を担当する。
//! Service層からUIビルダー依存を除去するために使用する。

use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, ButtonContent, ButtonStyleType, MessageContent, SelectMenuContent,
    SelectMenuOptionContent,
};
use rust_i18n::t;

/// 6属性の数
const SIX_ELEMENT_COUNT: usize = 6;

/// 属性情報
#[derive(Debug, Clone)]
pub struct ElementInfo {
    /// 属性ID
    pub id: i32,
    /// 属性名
    pub name: String,
    /// 絵文字
    pub emoji: &'static str,
}

/// 6属性の定義を取得する
pub fn get_six_elements() -> Vec<ElementInfo> {
    vec![
        ElementInfo {
            id: 1,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementFire.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "🔥",
        },
        ElementInfo {
            id: 2,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementWater.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "💧",
        },
        ElementInfo {
            id: 3,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementEarth.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "🌍",
        },
        ElementInfo {
            id: 4,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementWind.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "💨",
        },
        ElementInfo {
            id: 5,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementLight.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "✨",
        },
        ElementInfo {
            id: 6,
            name: t!(
                MessageTextId::AutoRecruitmentPresenterElementDark.as_str(),
                locale = "ja"
            )
            .to_string(),
            emoji: "🌑",
        },
    ]
}

/// 自動募集UIを担当するPresenter
///
/// クエスト選択、属性選択、参加ボタン等を生成する。
/// poise/serenity型は使用せず、ドメインモデルを返す。
pub struct AutoRecruitmentPresenter;

impl AutoRecruitmentPresenter {
    /// クエストメッセージを生成する（新規作成用）
    ///
    /// 6属性クエストの場合はセレクトメニュー、それ以外は参加ボタンを表示。
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    /// * `quest_name` - クエスト名
    /// * `is_six_element` - 6属性クエストかどうか
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_quest_message(
        guild_id: u64,
        quest_id: i32,
        quest_name: &str,
        is_six_element: bool,
    ) -> MessageContent {
        let action_row = if is_six_element {
            Self::create_element_select_menu(guild_id, quest_id)
        } else {
            Self::create_participation_button(guild_id, quest_id)
        };

        let content = format!("🎮 **{quest_name}**");

        MessageContent::new()
            .with_text(content)
            .with_component(action_row)
    }

    /// 参加ボタンを生成する（属性指定なしクエスト用）
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    ///
    /// # Returns
    ///
    /// ActionRowContent
    pub fn create_participation_button(guild_id: u64, quest_id: i32) -> ActionRowContent {
        let custom_id = format!("auto_quest_join:{guild_id}:{quest_id}");
        let label = t!(
            MessageTextId::AutoRecruitmentPresenterJoinButton.as_str(),
            locale = "ja"
        )
        .to_string();

        let button = ButtonContent::new(custom_id, label).with_style(ButtonStyleType::Primary);

        ActionRowContent::buttons(vec![button])
    }

    /// 属性選択セレクトメニューを生成する（6属性クエスト用）
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    ///
    /// # Returns
    ///
    /// ActionRowContent
    pub fn create_element_select_menu(guild_id: u64, quest_id: i32) -> ActionRowContent {
        let custom_id = format!("auto_quest_element:{guild_id}:{quest_id}");

        let elements = get_six_elements();
        let options: Vec<SelectMenuOptionContent> = elements
            .iter()
            .map(|element| {
                let label = format!("{} {}", element.emoji, element.name);
                SelectMenuOptionContent::new(label, element.id.to_string())
            })
            .collect();

        let select_menu = SelectMenuContent::string_select(custom_id, options)
            .with_placeholder(
                t!(
                    MessageTextId::AutoRecruitmentPresenterElementPlaceholder.as_str(),
                    locale = "ja"
                )
                .to_string(),
            )
            .with_min_values(0)
            .with_max_values(SIX_ELEMENT_COUNT as u8);

        ActionRowContent::select_menu(select_menu)
    }

    /// クエスト選択セレクトメニューを生成する
    ///
    /// 25件以上の場合は複数のセレクトメニューに分割。
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `quests` - (クエストID, クエスト名) のリスト
    /// * `max_values` - 最大選択数
    ///
    /// # Returns
    ///
    /// ActionRowContentのVec
    pub fn create_quest_select_menus(
        guild_id: u64,
        quests: &[(i32, String)],
        max_values: u8,
    ) -> Vec<ActionRowContent> {
        let mut action_rows = Vec::new();
        let chunk_size = 25;

        for (i, chunk) in quests.chunks(chunk_size).enumerate() {
            let options: Vec<SelectMenuOptionContent> = chunk
                .iter()
                .map(|(id, name)| SelectMenuOptionContent::new(name, id.to_string()))
                .collect();

            let effective_max = std::cmp::min(max_values, options.len() as u8);

            // チャンクごとにカスタムIDを変える
            let custom_id = if i == 0 {
                format!("auto_quest_select:{guild_id}")
            } else {
                format!("auto_quest_select:{guild_id}:{i}")
            };

            let select_menu = SelectMenuContent::string_select(custom_id, options)
                .with_placeholder(
                    t!(
                        MessageTextId::AutoRecruitmentPresenterQuestSelectPlaceholder.as_str(),
                        locale = "ja"
                    )
                    .to_string(),
                )
                .with_min_values(1)
                .with_max_values(effective_max);

            action_rows.push(ActionRowContent::select_menu(select_menu));
        }

        action_rows
    }

    /// クエスト選択メッセージを生成する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `quests` - (クエストID, クエスト名) のリスト
    /// * `max_values` - 最大選択数
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_quest_select_message(
        guild_id: u64,
        quests: &[(i32, String)],
        max_values: u8,
    ) -> MessageContent {
        let action_rows = Self::create_quest_select_menus(guild_id, quests, max_values);

        let mut message = MessageContent::new().with_text(
            t!(
                MessageTextId::AutoRecruitmentPresenterQuestSelectMessage.as_str(),
                locale = "ja"
            )
            .to_string(),
        );

        for action_row in action_rows {
            message = message.with_component(action_row);
        }

        message
    }

    /// 時間選択セレクトメニューを生成する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `time_slots` - (値, 表示名) のリスト
    ///
    /// # Returns
    ///
    /// ActionRowContent
    pub fn create_time_select_menu(
        guild_id: u64,
        time_slots: &[(String, String)],
    ) -> ActionRowContent {
        let custom_id = format!("auto_time_select:{guild_id}");

        let options: Vec<SelectMenuOptionContent> = time_slots
            .iter()
            .map(|(value, label)| SelectMenuOptionContent::new(label, value))
            .collect();

        let select_menu = SelectMenuContent::string_select(custom_id, options)
            .with_placeholder(
                t!(
                    MessageTextId::AutoRecruitmentPresenterTimeSelectPlaceholder.as_str(),
                    locale = "ja"
                )
                .to_string(),
            )
            .with_min_values(1)
            .with_max_values(1);

        ActionRowContent::select_menu(select_menu)
    }

    /// 設定完了メッセージを生成する
    ///
    /// # Arguments
    ///
    /// * `quest_name` - クエスト名
    /// * `time_display` - 選択された時間の表示
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_setup_complete_message(quest_name: &str, time_display: &str) -> MessageContent {
        use crate::types::discord::EmbedContent;

        let embed = EmbedContent::new()
            .with_title(
                t!(
                    MessageTextId::AutoRecruitmentPresenterSetupCompleteTitle.as_str(),
                    locale = "ja"
                )
                .to_string(),
            )
            .with_description(
                t!(
                    MessageTextId::AutoRecruitmentPresenterSetupCompleteDescription.as_str(),
                    locale = "ja"
                )
                .to_string(),
            )
            .with_color(0x00ff00)
            .with_field(
                t!(
                    MessageTextId::AutoRecruitmentPresenterSetupCompleteQuestField.as_str(),
                    locale = "ja"
                )
                .to_string(),
                quest_name,
                true,
            )
            .with_field(
                t!(
                    MessageTextId::AutoRecruitmentPresenterSetupCompleteTimeField.as_str(),
                    locale = "ja"
                )
                .to_string(),
                time_display,
                true,
            );

        MessageContent::new().with_embed(embed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_six_elements() {
        let elements = get_six_elements();
        assert_eq!(elements.len(), 6);
        assert_eq!(elements[0].name, "火属性");
        assert_eq!(elements[5].name, "闇属性");
    }

    #[test]
    fn test_create_quest_message_with_button() {
        let message =
            AutoRecruitmentPresenter::create_quest_message(12345, 1, "テストクエスト", false);

        assert!(message.text.as_ref().unwrap().contains("テストクエスト"));
        assert_eq!(message.components.len(), 1);
    }

    #[test]
    fn test_create_quest_message_with_select_menu() {
        let message =
            AutoRecruitmentPresenter::create_quest_message(12345, 1, "6属性クエスト", true);

        assert!(message.text.as_ref().unwrap().contains("6属性クエスト"));
        assert_eq!(message.components.len(), 1);
    }

    #[test]
    fn test_create_participation_button() {
        let action_row = AutoRecruitmentPresenter::create_participation_button(12345, 1);

        assert_eq!(action_row.components.len(), 1);
    }

    #[test]
    fn test_create_element_select_menu() {
        let action_row = AutoRecruitmentPresenter::create_element_select_menu(12345, 1);

        assert_eq!(action_row.components.len(), 1);
    }

    #[test]
    fn test_create_quest_select_menus_single() {
        let quests = vec![
            (1, "テスト1".to_string()),
            (2, "テスト2".to_string()),
            (3, "テスト3".to_string()),
        ];

        let action_rows = AutoRecruitmentPresenter::create_quest_select_menus(12345, &quests, 3);
        assert_eq!(action_rows.len(), 1);
    }

    #[test]
    fn test_create_quest_select_menus_multiple() {
        // 30件のクエストを作成
        let quests: Vec<(i32, String)> = (1..=30).map(|i| (i, format!("テスト{i}"))).collect();

        let action_rows = AutoRecruitmentPresenter::create_quest_select_menus(12345, &quests, 25);
        // 30件なので2つのセレクトメニューに分割
        assert_eq!(action_rows.len(), 2);
    }

    #[test]
    fn test_create_quest_select_message() {
        let quests = vec![(1, "テスト1".to_string()), (2, "テスト2".to_string())];

        let message = AutoRecruitmentPresenter::create_quest_select_message(12345, &quests, 2);

        assert!(message.text.as_ref().unwrap().contains("クエスト選択"));
        assert_eq!(message.components.len(), 1);
    }

    #[test]
    fn test_create_time_select_menu() {
        let time_slots = vec![
            ("21:00".to_string(), "21:00".to_string()),
            ("22:00".to_string(), "22:00".to_string()),
        ];

        let action_row = AutoRecruitmentPresenter::create_time_select_menu(12345, &time_slots);

        assert_eq!(action_row.components.len(), 1);
    }

    #[test]
    fn test_create_setup_complete_message() {
        let message =
            AutoRecruitmentPresenter::create_setup_complete_message("テストクエスト", "21:00");

        assert_eq!(message.embeds.len(), 1);
        assert_eq!(message.embeds[0].title, Some("設定完了".to_string()));
    }
}
