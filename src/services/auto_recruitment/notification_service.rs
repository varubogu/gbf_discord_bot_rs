//! 自動募集通知サービス
//!
//! マッチング成功時の通知メッセージを作成・送信するサービス

use crate::types::Result;
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateActionRow, CreateEmbed, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditMessage, Http, Message,
};
use std::sync::Arc;
use tracing::{debug, error, info};

/// 自動募集通知サービス
pub struct AutoRecruitmentNotificationService;

impl AutoRecruitmentNotificationService {
    pub fn new() -> Self {
        Self
    }

    /// マッチング成功通知をマッチングチャンネルに投稿
    pub async fn notify_match(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        participants: &[u64],
        quest_candidates: &[(i32, String)], // (quest_id, quest_name)
        month: i32,
        day: i32,
        hour: i32,
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            participant_count = participants.len(),
            quest_count = quest_candidates.len(),
            month,
            day,
            hour,
            "マッチング成功通知を送信します"
        );

        let channel = ChannelId::new(channel_id);

        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{}>", id)).collect();

        // Embed作成
        let embed = CreateEmbed::new()
            .title("🎮 マッチング成功！")
            .description(format!(
                "**日時**: {}月{}日 {}:00\n\n**参加者**: {}\n\n以下のクエスト候補から選択してください。",
                month,
                day,
                hour,
                participant_mentions.join(", ")
            ))
            .color(0x00ff00);

        // クエスト選択セレクトメニュー作成
        let mut options: Vec<CreateSelectMenuOption> = quest_candidates
            .iter()
            .map(|(id, name)| CreateSelectMenuOption::new(name, id.to_string()))
            .collect();

        // 「何でも良い」オプションを追加
        options.push(CreateSelectMenuOption::new("何でも良い", "any"));

        let select_menu = CreateSelectMenu::new(
            format!("auto_vote:{}", matched_id),
            CreateSelectMenuKind::String { options },
        )
        .placeholder("クエストを選択してください");

        let action_row = CreateActionRow::SelectMenu(select_menu);

        let message = CreateMessage::new()
            .content(participant_mentions.join(" "))
            .embed(embed)
            .components(vec![action_row]);

        let sent_message = channel.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id, "マッチング通知の送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("マッチング通知の送信に失敗しました: {}", e),
            }
        })?;

        info!(
            channel_id,
            message_id = sent_message.id.get(),
            "マッチング成功通知を送信しました"
        );

        Ok(sent_message)
    }

    /// 参加者追加時にメッセージを編集
    pub async fn update_match_notification(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        message_id: u64,
        participants: &[u64],
        quest_candidates: &[(i32, String)],
        month: i32,
        day: i32,
        hour: i32,
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            message_id,
            participant_count = participants.len(),
            "マッチング通知を更新します"
        );

        let channel = ChannelId::new(channel_id);

        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{}>", id)).collect();

        // Embed作成
        let embed = CreateEmbed::new()
            .title("🎮 マッチング成功！")
            .description(format!(
                "**日時**: {}月{}日 {}:00\n\n**参加者**: {}\n\n以下のクエスト候補から選択してください。",
                month,
                day,
                hour,
                participant_mentions.join(", ")
            ))
            .color(0x00ff00);

        // クエスト選択セレクトメニュー作成
        let mut options: Vec<CreateSelectMenuOption> = quest_candidates
            .iter()
            .map(|(id, name)| CreateSelectMenuOption::new(name, id.to_string()))
            .collect();

        options.push(CreateSelectMenuOption::new("何でも良い", "any"));

        let select_menu = CreateSelectMenu::new(
            format!("auto_vote:{}", matched_id),
            CreateSelectMenuKind::String { options },
        )
        .placeholder("クエストを選択してください");

        let action_row = CreateActionRow::SelectMenu(select_menu);

        let edit_message = EditMessage::new()
            .content(participant_mentions.join(" "))
            .embed(embed)
            .components(vec![action_row]);

        let mut message = channel
            .message(http, serenity::MessageId::new(message_id))
            .await
            .map_err(|e| {
                error!(error = %e, channel_id, message_id, "メッセージの取得に失敗しました");
                crate::types::AppError::Business {
                    message: format!("メッセージの取得に失敗しました: {}", e),
                }
            })?;

        message.edit(http, edit_message).await.map_err(|e| {
            error!(error = %e, channel_id, message_id, "メッセージの編集に失敗しました");
            crate::types::AppError::Business {
                message: format!("メッセージの編集に失敗しました: {}", e),
            }
        })?;

        info!(channel_id, message_id, "マッチング通知を更新しました");
        Ok(message)
    }

    /// 再投票メッセージを送信（元メッセージに返信）
    pub async fn send_revote_message(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        reply_to_message_id: u64,
        participants: &[u64],
        tie_quest_ids: &[(i32, String)],
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            reply_to_message_id, "再投票メッセージを送信します"
        );

        let channel = ChannelId::new(channel_id);

        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{}>", id)).collect();

        // Embed作成
        let embed = CreateEmbed::new()
            .title("🔄 再投票が必要です")
            .description(format!(
                "同数投票のため、以下のクエストから再度選択してください。\n\n{}",
                participant_mentions.join(" ")
            ))
            .color(0xffaa00);

        // 同数だったクエストのみ選択肢に
        let mut options: Vec<CreateSelectMenuOption> = tie_quest_ids
            .iter()
            .map(|(id, name)| CreateSelectMenuOption::new(name, id.to_string()))
            .collect();

        options.push(CreateSelectMenuOption::new("何でも良い", "any"));

        let select_menu = CreateSelectMenu::new(
            format!("auto_vote:{}", matched_id),
            CreateSelectMenuKind::String { options },
        )
        .placeholder("クエストを選択してください");

        let action_row = CreateActionRow::SelectMenu(select_menu);

        let message = CreateMessage::new()
            .content(participant_mentions.join(" "))
            .embed(embed)
            .components(vec![action_row])
            .reference_message((channel, serenity::MessageId::new(reply_to_message_id)));

        let sent_message = channel.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id, "再投票メッセージの送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("再投票メッセージの送信に失敗しました: {}", e),
            }
        })?;

        info!(
            channel_id,
            message_id = sent_message.id.get(),
            "再投票メッセージを送信しました"
        );

        Ok(sent_message)
    }

    /// クエスト決定通知を送信
    pub async fn notify_quest_decided(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        participants: &[u64],
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Message> {
        debug!(channel_id, quest_name, "クエスト決定通知を送信します");

        let channel = ChannelId::new(channel_id);

        // 参加者メンション
        let participant_mentions: Vec<String> =
            participants.iter().map(|id| format!("<@{}>", id)).collect();

        let embed = CreateEmbed::new()
            .title("✅ クエストが決定しました！")
            .description(format!(
                "**クエスト**: {}\n**日時**: {}月{}日 {}:00\n\n募集を作成しています...",
                quest_name, month, day, hour
            ))
            .color(0x00aaff);

        let message = CreateMessage::new()
            .content(participant_mentions.join(" "))
            .embed(embed);

        let sent_message = channel.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id, "クエスト決定通知の送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("クエスト決定通知の送信に失敗しました: {}", e),
            }
        })?;

        info!(channel_id, quest_name, "クエスト決定通知を送信しました");
        Ok(sent_message)
    }
}

impl Default for AutoRecruitmentNotificationService {
    fn default() -> Self {
        Self::new()
    }
}
