use crate::models::entities::{message_texts, notifications};
use crate::repository::database::schedule::NotificationRepository;
use crate::types::Result;
use chrono::{DateTime, Duration, Utc};
use poise::serenity_prelude::{self as serenity, ChannelId, CreateMessage, Http, ReactionType};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 通知実行サービス
pub struct NotificationService {
    db: DatabaseConnection,
    notification_repo: NotificationRepository,
    http: Arc<Http>,
}

impl NotificationService {
    pub fn new(db: DatabaseConnection, http: Arc<Http>) -> Self {
        let notification_repo = NotificationRepository::new(db.clone());
        Self {
            db,
            notification_repo,
            http,
        }
    }

    /// スケジュール通知を実行
    /// 現在時刻から次の10秒後までの通知を取得して実行
    pub async fn execute_scheduled_notifications(&self) -> Result<()> {
        let now = Utc::now();
        let next_tick = now + Duration::seconds(10);

        debug!(
            from = %now,
            to = %next_tick,
            "スケジュール通知の実行を開始します"
        );

        // 実行対象の通知を取得
        let notifications = self
            .notification_repo
            .find_by_datetime_range(now, next_tick)
            .await?;

        if notifications.is_empty() {
            debug!("実行対象の通知はありません");
            return Ok(());
        }

        info!(count = notifications.len(), "通知を実行します");

        let mut success_count = 0;
        let mut error_count = 0;

        for notification in notifications {
            match self.send_notification(&notification).await {
                Ok(_) => {
                    success_count += 1;
                    debug!(
                        notification_id = notification.id,
                        guild_id = notification.guild_id,
                        "通知を送信しました"
                    );
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        error = %e,
                        notification_id = notification.id,
                        guild_id = notification.guild_id,
                        channel_id = notification.channel_id,
                        "通知の送信に失敗しました"
                    );
                }
            }
        }

        info!(
            success = success_count,
            error = error_count,
            "スケジュール通知の実行が完了しました"
        );

        Ok(())
    }

    /// 個別通知を送信
    async fn send_notification(&self, notification: &notifications::Model) -> Result<()> {
        debug!(
            notification_id = notification.id,
            message_text_id = %notification.message_text_id,
            "通知メッセージを取得します"
        );

        // メッセージテキストを取得
        let message_text = self.get_message_text(&notification.message_text_id).await?;

        // チャンネルにメッセージを送信
        let channel_id = ChannelId::new(notification.channel_id as u64);

        let message = CreateMessage::new().content(&message_text.message_jp);

        match channel_id.send_message(&self.http, message).await {
            Ok(sent_message) => {
                info!(
                    notification_id = notification.id,
                    guild_id = notification.guild_id,
                    channel_id = notification.channel_id,
                    message_id = sent_message.id.get(),
                    "通知を送信しました"
                );

                // TODO: リアクション追加機能（event_schedule_detailsのreactionsフィールドを使用）
                // 現状の実装ではreactionsフィールドにアクセスできないため、将来的に実装

                Ok(())
            }
            Err(e) => {
                error!(
                    error = %e,
                    notification_id = notification.id,
                    channel_id = notification.channel_id,
                    "メッセージの送信に失敗しました"
                );
                Err(e.into())
            }
        }
    }

    /// メッセージテキストを取得
    async fn get_message_text(&self, message_text_id: &str) -> Result<message_texts::Model> {
        let message_text = message_texts::Entity::find()
            .filter(message_texts::Column::Id.eq(message_text_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    message_text_id = %message_text_id,
                    "メッセージテキストの取得に失敗しました"
                );
                e
            })?;

        message_text.ok_or_else(|| {
            warn!(
                message_text_id = %message_text_id,
                "メッセージテキストが見つかりません"
            );
            crate::types::AppError::NotFound(format!(
                "メッセージテキストが見つかりません: {}",
                message_text_id
            ))
        })
    }

    /// リアクションを追加（将来的な実装用）
    #[allow(dead_code)]
    async fn add_reactions(
        &self,
        channel_id: ChannelId,
        message_id: serenity::MessageId,
        reactions: &str,
    ) -> Result<()> {
        if reactions.is_empty() {
            return Ok(());
        }

        for reaction_str in reactions.split(',') {
            let trimmed = reaction_str.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Unicode絵文字またはカスタム絵文字をパース
            let reaction_type = if let Ok(emoji_id) = trimmed.parse::<u64>() {
                ReactionType::Custom {
                    animated: false,
                    id: serenity::EmojiId::new(emoji_id),
                    name: None,
                }
            } else {
                ReactionType::Unicode(trimmed.to_string())
            };

            if let Err(e) = self
                .http
                .create_reaction(channel_id, message_id, &reaction_type)
                .await
            {
                warn!(
                    error = %e,
                    reaction = %trimmed,
                    "リアクションの追加に失敗しました"
                );
            }
        }

        Ok(())
    }
}
