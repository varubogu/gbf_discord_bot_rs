use crate::models::entities::{battle_recruitments, message_texts, notifications};
use crate::repository::database::schedule::{NotificationRelBattleRecruitmentRepository, NotificationRepository};
use crate::types::Result;
use chrono::Utc;
use poise::serenity_prelude::{ChannelId, CreateMessage, Http, MessageId};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 通知実行サービス
pub struct NotificationService {
    db: DatabaseConnection,
    notification_repo: NotificationRepository,
    rel_repo: NotificationRelBattleRecruitmentRepository,
    http: Arc<Http>,
}

impl NotificationService {
    pub fn new(db: DatabaseConnection, http: Arc<Http>) -> Self {
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelBattleRecruitmentRepository::new();
        Self {
            db,
            notification_repo,
            rel_repo,
            http,
        }
    }

    /// スケジュール通知を実行
    /// last_process_times.execute_timeから現在時刻までの通知を取得して実行
    /// last_process_timesが存在しない場合は現在時刻のみを対象とする
    pub async fn execute_scheduled_notifications(
        &self,
        last_process_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        let now = Utc::now();

        // 前回実行時刻から現在時刻までの範囲を設定
        // 前回実行時刻が存在しない場合は現在時刻のみを対象
        let from = last_process_time.unwrap_or(now);

        debug!(
            from = %from,
            to = %now,
            "スケジュール通知の実行を開始します"
        );

        // 実行対象の通知を取得
        let notifications = self
            .notification_repo
            .find_by_datetime_range(&self.db, from, now)
            .await?;

        if notifications.is_empty() {
            debug!("実行対象の通知はありません");
            return Ok(());
        }

        info!(count = notifications.len(), "通知を実行します");

        let mut success_count = 0;
        let mut error_count = 0;

        for notification in notifications {
            // 通知を送信
            let send_result = self.send_notification(&notification).await;

            // 送信結果に関わらず、トランザクションでis_sentフラグを更新
            // 送信成功した場合のみフラグを立てる
            if send_result.is_ok() {
                let txn = self.db.begin().await?;
                match self.notification_repo.mark_as_sent_with_txn(&txn, notification.id).await {
                    Ok(_) => {
                        txn.commit().await?;
                        success_count += 1;
                        debug!(
                            notification_id = notification.id,
                            guild_id = notification.guild_id,
                            "通知を送信し、送信済みとしてマークしました"
                        );
                    }
                    Err(e) => {
                        txn.rollback().await?;
                        error_count += 1;
                        error!(
                            error = %e,
                            notification_id = notification.id,
                            "is_sentフラグの更新に失敗しました（次回再送されます）"
                        );
                    }
                }
            } else {
                error_count += 1;
                error!(
                    error = %send_result.unwrap_err(),
                    notification_id = notification.id,
                    guild_id = notification.guild_id,
                    channel_id = notification.channel_id,
                    "通知の送信に失敗しました（次回リトライされます）"
                );
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

        // リレーションを確認してマルチ募集通知かどうかを判定
        if let Some(rel) = self.rel_repo.find_by_notification_id(&self.db, notification.id).await? {
            // マルチ募集通知の場合
            info!(
                notification_id = notification.id,
                recruit_id = rel.recruit_id,
                "マルチ募集通知を送信します"
            );
            self.send_recruitment_notification(notification, rel.recruit_id).await?;
        } else {
            // 通常の通知の場合
            self.send_normal_notification(notification).await?;
        }

        Ok(())
    }

    /// マルチ募集通知を送信（募集メッセージへの返信とメンション）
    async fn send_recruitment_notification(
        &self,
        notification: &notifications::Model,
        recruit_id: i32,
    ) -> Result<()> {
        // 募集情報を取得
        let recruitment = battle_recruitments::Entity::find_by_id(recruit_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!("募集ID {} が見つかりません", recruit_id))
            })?;

        // メッセージテキストを取得
        let message_text = self.get_message_text(&notification.message_text_id).await?;

        // チャンネルとメッセージIDを取得（i64 → u64にキャスト）
        let channel_id = ChannelId::new(recruitment.channel_id as u64);
        let message_id = MessageId::new(recruitment.message_id as u64);

        // 募集メッセージを取得してリアクション参加者を取得
        let recruit_message = channel_id.message(&self.http, message_id).await?;

        // 全リアクションから参加者のユーザーIDを収集
        let mut participant_ids = std::collections::HashSet::new();

        for reaction in &recruit_message.reactions {
            // リアクションしたユーザーを取得（最大100人）
            let users = channel_id
                .reaction_users(&self.http, message_id, reaction.reaction_type.clone(), Some(100), None)
                .await?;

            for user in users {
                // Botは除外
                if !user.bot {
                    participant_ids.insert(user.id);
                }
            }
        }

        // 参加者数を保存（メンション作成でmoveされる前に）
        let participants_count = participant_ids.len();

        // 参加者メンションを作成
        let mut mentions = String::new();
        for user_id in participant_ids {
            mentions.push_str(&format!("<@{}> ", user_id));
        }

        // 通知メッセージを作成（募集メッセージへの返信）
        let content = if mentions.is_empty() {
            message_text.message_jp.clone()
        } else {
            format!("{}\n{}", mentions, message_text.message_jp)
        };

        let message = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id));

        match channel_id.send_message(&self.http, message).await {
            Ok(sent_message) => {
                info!(
                    notification_id = notification.id,
                    recruit_id = recruit_id,
                    channel_id = recruitment.channel_id,
                    message_id = sent_message.id.get(),
                    participants_count = participants_count,
                    "マルチ募集通知を送信しました"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    error = %e,
                    notification_id = notification.id,
                    channel_id = recruitment.channel_id,
                    "マルチ募集通知の送信に失敗しました"
                );
                Err(e.into())
            }
        }
    }

    /// 通常の通知を送信
    async fn send_normal_notification(&self, notification: &notifications::Model) -> Result<()> {
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
}
