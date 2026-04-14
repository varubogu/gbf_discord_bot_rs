//! 通知プレゼンター
//!
//! 各種通知メッセージ（マッチング成功、出発通知、解散通知等）の作成を担当する。
//! Service層からUIビルダー依存を除去するために使用する。

use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, EmbedContent, MessageContent, SelectMenuContent, SelectMenuOptionContent,
};
use crate::utils::datetime_display::weekday_token_for_month_day_jst;
use rust_i18n::t;

/// 通知表示を担当するPresenter
///
/// マッチング成功通知、出発通知、解散通知等を生成する。
/// poise/serenity型は使用せず、ドメインモデルを返す。
pub struct NotificationPresenter;

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

impl NotificationPresenter {
    /// マッチング成功通知メッセージを生成する
    ///
    /// # Arguments
    ///
    /// * `participants` - 参加者のユーザーID一覧
    /// * `quest_candidates` - (クエストID, クエスト名) の候補一覧
    /// * `month` - 月
    /// * `day` - 日
    /// * `hour` - 時
    /// * `matched_id` - マッチングID
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_match_notification(
        participants: &[u64],
        quest_candidates: &[(i32, String)],
        month: i32,
        day: i32,
        hour: i32,
        matched_id: i32,
    ) -> MessageContent {
        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{id}>")).collect();

        // Embed作成
        let participants_str = participant_mentions.join(", ");
        let embed = EmbedContent::new()
            .with_title(localized_ja(MessageTextId::NotificationPresenterMatchTitle))
            .with_description(localized_ja_with_params(
                MessageTextId::NotificationPresenterMatchDescription,
                &[
                    ("month", month.to_string()),
                    ("day", day.to_string()),
                    ("weekday", Self::weekday_token(month, day)),
                    ("hour", hour.to_string()),
                    ("participants_str", participants_str),
                ],
            ))
            .with_color(0x00ff00);

        // クエスト選択セレクトメニュー作成
        let action_row = Self::create_quest_vote_select_menu(quest_candidates, matched_id, true);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
            .with_component(action_row)
    }

    /// 自動マッチング完了通知メッセージを生成する
    pub fn create_auto_matching_notification(
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
        users: &[(u64, Option<i32>)],
        recruitment_url: Option<&str>,
    ) -> MessageContent {
        let participant_mentions: Vec<String> = users
            .iter()
            .map(|(user_id, _)| format!("<@{user_id}>"))
            .collect();
        let participants_str = participant_mentions.join("\n");
        let element_info = Self::create_auto_matching_element_info(users);
        let status_message = if recruitment_url.is_some() {
            localized_ja(MessageTextId::NotificationPresenterAutoMatchingStatusCreated)
        } else {
            localized_ja(MessageTextId::NotificationPresenterAutoMatchingStatusCreating)
        };

        let mut embed = EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::NotificationPresenterAutoMatchingTitle,
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::NotificationPresenterAutoMatchingDescription,
                &[
                    ("quest_name", quest_name.to_string()),
                    ("month", month.to_string()),
                    ("day", day.to_string()),
                    ("weekday", Self::weekday_token(month, day)),
                    ("hour", hour.to_string()),
                    ("participants_str", participants_str),
                    ("element_info", element_info),
                    ("status_text", status_message),
                ],
            ))
            .with_color(0x00ff00);

        if let Some(url) = recruitment_url {
            embed = embed.with_field(
                localized_ja(MessageTextId::NotificationPresenterAutoMatchingRecruitmentFieldTitle),
                url,
                false,
            );
        }

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
    }

    /// 再投票通知メッセージを生成する
    ///
    /// # Arguments
    ///
    /// * `participants` - 参加者のユーザーID一覧
    /// * `tie_quests` - (クエストID, クエスト名) の同票クエスト一覧
    /// * `matched_id` - マッチングID
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_revote_notification(
        participants: &[u64],
        tie_quests: &[(i32, String)],
        matched_id: i32,
    ) -> MessageContent {
        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{id}>")).collect();

        // Embed作成
        let participants_str = participant_mentions.join(" ");
        let embed = EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::NotificationPresenterRevoteTitle,
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::NotificationPresenterRevoteDescription,
                &[("participants_str", participants_str)],
            ))
            .with_color(0xffaa00);

        // セレクトメニュー（同票クエストのみ）
        let action_row = Self::create_quest_vote_select_menu(tie_quests, matched_id, true);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
            .with_component(action_row)
    }

    /// クエスト決定通知メッセージを生成する
    ///
    /// # Arguments
    ///
    /// * `participants` - 参加者のユーザーID一覧
    /// * `quest_name` - 決定したクエスト名
    /// * `month` - 月
    /// * `day` - 日
    /// * `hour` - 時
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_quest_decided_notification(
        participants: &[u64],
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
    ) -> MessageContent {
        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{id}>")).collect();

        let embed = EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::NotificationPresenterQuestDecidedTitle,
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::NotificationPresenterQuestDecidedDescription,
                &[
                    ("quest_name", quest_name.to_string()),
                    ("month", month.to_string()),
                    ("day", day.to_string()),
                    ("weekday", Self::weekday_token(month, day)),
                    ("hour", hour.to_string()),
                ],
            ))
            .with_color(0x00aaff);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
    }

    /// クエスト投票用セレクトメニューを生成する
    ///
    /// # Arguments
    ///
    /// * `quests` - (クエストID, クエスト名) の一覧
    /// * `matched_id` - マッチングID
    /// * `include_any` - 「何でも良い」オプションを含めるか
    ///
    /// # Returns
    ///
    /// ActionRowContent
    fn create_quest_vote_select_menu(
        quests: &[(i32, String)],
        matched_id: i32,
        include_any: bool,
    ) -> ActionRowContent {
        let mut options: Vec<SelectMenuOptionContent> = quests
            .iter()
            .map(|(id, name)| SelectMenuOptionContent::new(name, id.to_string()))
            .collect();

        if include_any {
            options.push(SelectMenuOptionContent::new(
                localized_ja(MessageTextId::NotificationPresenterVoteOptionAny),
                "any",
            ));
        }

        let custom_id = format!("auto_vote:{matched_id}");
        let select_menu = SelectMenuContent::string_select(custom_id, options).with_placeholder(
            localized_ja(MessageTextId::NotificationPresenterVotePlaceholder),
        );

        ActionRowContent::select_menu(select_menu)
    }

    /// 自動マッチング通知用の属性表示を生成する
    fn create_auto_matching_element_info(users: &[(u64, Option<i32>)]) -> String {
        let has_elements = users
            .iter()
            .any(|(_, style)| style.is_some() && *style != Some(0));

        if !has_elements {
            return String::new();
        }

        let element_names = [
            (1, "火"),
            (2, "水"),
            (3, "土"),
            (4, "風"),
            (5, "光"),
            (6, "闇"),
        ];

        let elements: Vec<String> = users
            .iter()
            .filter_map(|(user_id, style)| {
                style.and_then(|battle_style_id| {
                    if battle_style_id > 0 {
                        element_names
                            .iter()
                            .find(|(id, _)| *id == battle_style_id)
                            .map(|(_, name)| format!("<@{user_id}>: {name}"))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if elements.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n**{}**:\n{}",
                localized_ja(MessageTextId::NotificationPresenterAutoMatchingElementHeader),
                elements.join("\n"),
            )
        }
    }

    /// 月日からJST基準の曜日トークン（例: `(水)`）を生成する
    fn weekday_token(month: i32, day: i32) -> String {
        weekday_token_for_month_day_jst(month, day, "ja").unwrap_or_default()
    }

    /// 出発通知Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `quest_name` - クエスト名
    /// * `participants` - 参加者のユーザーID一覧
    /// * `is_five_minute_warning` - 5分前通知かどうか
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_departure_notification(
        quest_name: &str,
        participants: &[u64],
        is_five_minute_warning: bool,
    ) -> MessageContent {
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{id}>")).collect();

        let (title, description, color) = if is_five_minute_warning {
            (
                localized_ja(MessageTextId::NotificationPresenterDepartureFiveMinuteTitle),
                localized_ja_with_params(
                    MessageTextId::NotificationPresenterDepartureFiveMinuteDescription,
                    &[("quest_name", quest_name.to_string())],
                ),
                0xffaa00,
            )
        } else {
            (
                localized_ja(MessageTextId::NotificationPresenterDepartureNowTitle),
                localized_ja_with_params(
                    MessageTextId::NotificationPresenterDepartureNowDescription,
                    &[("quest_name", quest_name.to_string())],
                ),
                0x00ff00,
            )
        };

        let embed = EmbedContent::new()
            .with_title(title)
            .with_description(description)
            .with_color(color);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
    }

    /// 解散通知Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `quest_name` - クエスト名
    /// * `participants` - 参加者のユーザーID一覧
    ///
    /// # Returns
    ///
    /// MessageContent
    pub fn create_dissolution_notification(
        quest_name: &str,
        participants: &[u64],
    ) -> MessageContent {
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{id}>")).collect();

        let embed = EmbedContent::new()
            .with_title(localized_ja(
                MessageTextId::NotificationPresenterDissolutionTitle,
            ))
            .with_description(localized_ja_with_params(
                MessageTextId::NotificationPresenterDissolutionDescription,
                &[("quest_name", quest_name.to_string())],
            ))
            .with_color(0x808080);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
    }

    /// エラー通知Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `title` - エラータイトル
    /// * `message` - エラーメッセージ
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_error_embed(title: &str, message: &str) -> EmbedContent {
        EmbedContent::new()
            .with_title(format!("❌ {title}"))
            .with_description(message)
            .with_color(0xff0000)
    }

    /// 成功通知Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `title` - タイトル
    /// * `message` - メッセージ
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_success_embed(title: &str, message: &str) -> EmbedContent {
        EmbedContent::new()
            .with_title(format!("✅ {title}"))
            .with_description(message)
            .with_color(0x00ff00)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_match_notification() {
        let participants = vec![123456789, 987654321];
        let quests = vec![
            (1, "テストクエスト1".to_string()),
            (2, "テストクエスト2".to_string()),
        ];

        let message =
            NotificationPresenter::create_match_notification(&participants, &quests, 1, 15, 21, 1);

        assert!(message.text.as_ref().unwrap().contains("<@123456789>"));
        assert_eq!(message.embeds.len(), 1);
        assert!(
            message.embeds[0]
                .title
                .as_ref()
                .unwrap()
                .contains("マッチング成功")
        );
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("1月15日 (")
        );
        assert_eq!(message.components.len(), 1);
    }

    #[test]
    fn test_create_revote_notification() {
        let participants = vec![123456789];
        let tie_quests = vec![(1, "クエストA".to_string()), (2, "クエストB".to_string())];

        let message =
            NotificationPresenter::create_revote_notification(&participants, &tie_quests, 1);

        assert_eq!(message.embeds.len(), 1);
        assert!(message.embeds[0].title.as_ref().unwrap().contains("再投票"));
    }

    #[test]
    fn test_create_quest_decided_notification() {
        let participants = vec![123456789];

        let message = NotificationPresenter::create_quest_decided_notification(
            &participants,
            "決定クエスト",
            1,
            15,
            21,
        );

        assert_eq!(message.embeds.len(), 1);
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("決定クエスト")
        );
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("1月15日 (")
        );
    }

    #[test]
    fn test_create_auto_matching_notification_without_link() {
        let message = NotificationPresenter::create_auto_matching_notification(
            "テストクエスト",
            1,
            15,
            21,
            &[(123456789, Some(1)), (987654321, None)],
            None,
        );

        assert_eq!(message.embeds.len(), 1);
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("募集を作成しています")
        );
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("1月15日 (")
        );
        assert_eq!(message.embeds[0].fields.len(), 0);
    }

    #[test]
    fn test_create_auto_matching_notification_with_link() {
        let recruitment_url = "https://discord.com/channels/1/2/3";
        let message = NotificationPresenter::create_auto_matching_notification(
            "テストクエスト",
            1,
            15,
            21,
            &[(123456789, Some(1)), (987654321, Some(2))],
            Some(recruitment_url),
        );

        assert_eq!(message.embeds.len(), 1);
        assert!(
            message.embeds[0]
                .description
                .as_ref()
                .unwrap()
                .contains("募集を作成しました")
        );
        assert_eq!(message.embeds[0].fields.len(), 1);
        assert_eq!(message.embeds[0].fields[0].value, recruitment_url);
    }

    #[test]
    fn test_create_departure_notification_five_min() {
        let participants = vec![123456789];

        let message = NotificationPresenter::create_departure_notification(
            "テストクエスト",
            &participants,
            true,
        );

        assert!(
            message.embeds[0]
                .title
                .as_ref()
                .unwrap()
                .contains("まもなく")
        );
        assert_eq!(message.embeds[0].color, Some(0xffaa00));
    }

    #[test]
    fn test_create_departure_notification_now() {
        let participants = vec![123456789];

        let message = NotificationPresenter::create_departure_notification(
            "テストクエスト",
            &participants,
            false,
        );

        assert!(
            message.embeds[0]
                .title
                .as_ref()
                .unwrap()
                .contains("出発時刻")
        );
        assert_eq!(message.embeds[0].color, Some(0x00ff00));
    }

    #[test]
    fn test_create_dissolution_notification() {
        let participants = vec![123456789];

        let message =
            NotificationPresenter::create_dissolution_notification("テストクエスト", &participants);

        assert!(message.embeds[0].title.as_ref().unwrap().contains("解散"));
        assert_eq!(message.embeds[0].color, Some(0x808080));
    }

    #[test]
    fn test_create_error_embed() {
        let embed = NotificationPresenter::create_error_embed("エラー発生", "処理に失敗しました");

        assert!(embed.title.as_ref().unwrap().contains("❌"));
        assert_eq!(embed.color, Some(0xff0000));
    }

    #[test]
    fn test_create_success_embed() {
        let embed = NotificationPresenter::create_success_embed("成功", "処理が完了しました");

        assert!(embed.title.as_ref().unwrap().contains("✅"));
        assert_eq!(embed.color, Some(0x00ff00));
    }
}
