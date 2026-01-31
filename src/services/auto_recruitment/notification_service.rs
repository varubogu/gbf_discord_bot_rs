//! 自動募集通知サービス
//!
//! マッチング成功時の通知メッセージを作成・送信するサービス。
//! UIレイアウトは `NotificationPresenter` が担当し、本モジュールでは
//! プレゼンターのドメインモデルをserenityのビルダー型へ変換して送信する。

use crate::events::converters::{to_create_message, to_edit_message};
use crate::presenter::NotificationPresenter;
use crate::types::Result;
use poise::serenity_prelude::{self as serenity, ChannelId, Http, Message};
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

        // プレゼンターでメッセージを構築
        let message_content = NotificationPresenter::create_match_notification(
            participants,
            quest_candidates,
            month,
            day,
            hour,
            matched_id,
        );
        let message = to_create_message(&message_content);

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

        // プレゼンターで最新のメッセージ内容を構築
        let message_content = NotificationPresenter::create_match_notification(
            participants,
            quest_candidates,
            month,
            day,
            hour,
            matched_id,
        );
        let edit_message = to_edit_message(&message_content);

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

        // プレゼンターで再投票メッセージを構築
        let message_content = NotificationPresenter::create_revote_notification(
            participants,
            tie_quest_ids,
            matched_id,
        );
        let mut create_message = to_create_message(&message_content);
        create_message = create_message
            .reference_message((channel, serenity::MessageId::new(reply_to_message_id)));

        let sent_message = channel
            .send_message(http, create_message)
            .await
            .map_err(|e| {
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

        // プレゼンターでクエスト決定通知を構築
        let message_content = NotificationPresenter::create_quest_decided_notification(
            participants,
            quest_name,
            month,
            day,
            hour,
        );
        let message = to_create_message(&message_content);

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
