//! 通知プレゼンター
//!
//! 各種通知メッセージ（マッチング成功、出発通知、解散通知等）の作成を担当する。
//! Service層からUIビルダー依存を除去するために使用する。

use crate::types::discord::{
    ActionRowContent, EmbedContent, MessageContent, SelectMenuContent, SelectMenuOptionContent,
};

/// 通知表示を担当するPresenter
///
/// マッチング成功通知、出発通知、解散通知等を生成する。
/// poise/serenity型は使用せず、ドメインモデルを返す。
pub struct NotificationPresenter;

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
            .with_title("🎮 マッチング成功！")
            .with_description(format!(
                "**日時**: {month}月{day}日 {hour}:00\n\n**参加者**: {participants_str}\n\n以下のクエスト候補から選択してください。"
            ))
            .with_color(0x00ff00);

        // クエスト選択セレクトメニュー作成
        let action_row = Self::create_quest_vote_select_menu(quest_candidates, matched_id, true);

        MessageContent::new()
            .with_text(participant_mentions.join(" "))
            .with_embed(embed)
            .with_component(action_row)
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
            .with_title("🔄 再投票が必要です")
            .with_description(format!(
                "同数投票のため、以下のクエストから再度選択してください。\n\n{participants_str}"
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
            .with_title("✅ クエストが決定しました！")
            .with_description(format!(
                "**クエスト**: {quest_name}\n**日時**: {month}月{day}日 {hour}:00\n\n募集を作成しています..."
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
            options.push(SelectMenuOptionContent::new("何でも良い", "any"));
        }

        let custom_id = format!("auto_vote:{matched_id}");
        let select_menu = SelectMenuContent::string_select(custom_id, options)
            .with_placeholder("クエストを選択してください");

        ActionRowContent::select_menu(select_menu)
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
                "⏰ まもなく出発！",
                format!("「{quest_name}」の出発まであと5分です！\n準備はできていますか？"),
                0xffaa00,
            )
        } else {
            (
                "🚀 出発時刻です！",
                format!("「{quest_name}」の出発時刻になりました！\nよい狩りを！"),
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
            .with_title("📢 解散時刻です")
            .with_description(format!(
                "「{quest_name}」の解散時刻になりました。\nお疲れ様でした！"
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
