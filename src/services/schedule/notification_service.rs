use crate::models::entities::worker::{battle_recruitments, notifications};
use crate::repository::RecruitmentParticipantsRepository;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
use crate::repository::database::recruitment_participants_repository::SeaOrmRecruitmentParticipantsRepository;
use crate::repository::database::schedule::{
    SeaOrmNotificationRelBattleRecruitmentRepository, SeaOrmNotificationRepository,
};
use crate::repository::schedule::{NotificationRelBattleRecruitmentRepository, NotificationRepository};
use crate::repository::GuildSettingsRepository;
use crate::services::message::MessageService;
use crate::types::Result;
use crate::utils::discord_helper::send_message_with_optional_reply;
use chrono::Utc;
use poise::serenity_prelude::{ChannelId, CreateMessage, Http, MessageId};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// 通知実行サービス
/// - DatabaseConnection を保持しない
/// - すべてのDB操作はFacade層から渡されたトランザクション経由で実行する
pub struct NotificationService {
    notification_repo: SeaOrmNotificationRepository,
    rel_repo: SeaOrmNotificationRelBattleRecruitmentRepository,
    guild_timezone_repo: SeaOrmGuildSettingsRepository,
    message_service: MessageService,
    http: Arc<Http>,
}

impl NotificationService {
    pub fn new(http: Arc<Http>) -> Self {
        let notification_repo = SeaOrmNotificationRepository::new();
        let rel_repo = SeaOrmNotificationRelBattleRecruitmentRepository::new();
        let guild_timezone_repo = SeaOrmGuildSettingsRepository::new();
        let message_service = MessageService::new();
        Self {
            notification_repo,
            rel_repo,
            guild_timezone_repo,
            message_service,
            http,
        }
    }

    /// スケジュール通知を実行
    /// last_process_times.execute_timeから現在時刻までの通知を取得して実行
    /// last_process_timesが存在しない場合は現在時刻のみを対象とする
    ///
    /// 注意: トランザクション境界はFacade層が管理する
    pub async fn execute_scheduled_notifications(
        &self,
        txn: &DatabaseTransaction,
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

        // 実行対象の通知を取得（トランザクション内）
        let notifications = self
            .notification_repo
            .find_by_datetime_range_with_txn(txn, from, now)
            .await?;

        if notifications.is_empty() {
            debug!("実行対象の通知はありません");
            return Ok(());
        }

        info!(count = notifications.len(), "通知を実行します");

        let mut success_count = 0;

        for notification in notifications {
            // 通知を送信してis_sentフラグを更新
            match self.send_single_notification(txn, &notification).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error!(
                        error = %e,
                        notification_id = notification.id,
                        guild_id = notification.guild_id,
                        channel_id = notification.channel_id,
                        "通知の送信に失敗しました（次回リトライされます）"
                    );
                    // 送信失敗はロールバック対象にする（再送のため）
                    return Err(e);
                }
            }
        }

        info!(
            success = success_count,
            "スケジュール通知の実行が完了しました"
        );

        Ok(())
    }

    /// ギルドのロケールを取得
    /// 未設定の場合はデフォルト（ja）を返す
    async fn get_guild_locale(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<String> {
        match self
            .guild_timezone_repo
            .find_by_guild_id_with_txn(txn, guild_id)
            .await?
        {
            Some(settings) => Ok(settings.locale),
            None => {
                // 未設定の場合はデフォルト（ja）を返す
                debug!(
                    guild_id = guild_id,
                    "ロケール未設定のため、デフォルト（ja）を使用します"
                );
                Ok("ja".to_string())
            }
        }
    }

    /// 個別通知を送信してis_sentフラグを更新
    /// SchedulerManager用の公開メソッド
    pub async fn send_single_notification(
        &self,
        txn: &DatabaseTransaction,
        notification: &notifications::Model,
    ) -> Result<()> {
        // 通知を送信
        self.send_notification_internal(txn, notification).await?;

        // is_sentフラグを更新
        self.notification_repo
            .mark_as_sent_with_txn(txn, notification.id)
            .await?;

        debug!(
            notification_id = notification.id,
            guild_id = notification.guild_id,
            "通知を送信し、送信済みとしてマークしました"
        );

        Ok(())
    }

    /// 個別通知を送信（内部用）
    async fn send_notification_internal(
        &self,
        txn: &DatabaseTransaction,
        notification: &notifications::Model,
    ) -> Result<()> {
        debug!(
            notification_id = notification.id,
            message_text_id = %notification.message_text_id,
            "通知メッセージを取得します"
        );

        // リレーションを確認してマルチ募集通知かどうかを判定
        if let Some(rel) = self
            .rel_repo
            .find_by_notification_id(txn, notification.id)
            .await?
        {
            // マルチ募集通知の場合
            info!(
                notification_id = notification.id,
                recruit_id = rel.recruit_id,
                "マルチ募集通知を送信します"
            );
            self.send_recruitment_notification(txn, notification, rel.recruit_id)
                .await?;
        } else {
            // 通常の通知の場合
            self.send_normal_notification(txn, notification).await?;
        }

        Ok(())
    }

    /// マルチ募集通知を送信（募集メッセージへの返信とメンション）
    async fn send_recruitment_notification(
        &self,
        txn: &DatabaseTransaction,
        notification: &notifications::Model,
        recruit_id: i32,
    ) -> Result<()> {
        // 募集情報を取得
        let recruitment = battle_recruitments::Entity::find()
            .filter(battle_recruitments::Column::Id.eq(recruit_id))
            .one(txn)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!("募集ID {recruit_id} が見つかりません"))
            })?;

        // ギルドのロケールを取得
        let locale = self.get_guild_locale(txn, notification.guild_id).await?;

        // メッセージテキストを新しいメッセージサービスで取得
        // DatabaseTransactionはConnectionTraitを実装しているので直接渡せる
        let message_text = self
            .message_service
            .get_message(
                txn,
                &notification.message_text_id,
                HashMap::new(),
                Some(notification.guild_id),
                Some(&locale),
            )
            .await?;

        // チャンネルとメッセージIDを取得（i64 → u64にキャスト）
        let channel_id = ChannelId::new(recruitment.channel_id as u64);
        let message_id = MessageId::new(recruitment.message_id as u64);

        // recruitment_participantsテーブルから参加者を取得
        let participants_repo = SeaOrmRecruitmentParticipantsRepository::new();
        let participant_user_ids = participants_repo
            .get_all_participant_user_ids(txn, recruit_id)
            .await?;

        // 参加者数を保存
        let participants_count = participant_user_ids.len();

        // 参加者メンションを作成
        let mut mentions = String::new();
        for user_id in participant_user_ids {
            mentions.push_str(&format!("<@{user_id}> "));
        }

        // 通知メッセージを作成（募集メッセージへの返信）
        let content = if mentions.is_empty() {
            message_text.clone()
        } else {
            format!("{mentions}\n{message_text}")
        };

        // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
        match send_message_with_optional_reply(
            &self.http,
            channel_id,
            message_id,
            content,
            Some("スケジュール通知".to_string()),
        )
        .await
        {
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
    async fn send_normal_notification(
        &self,
        txn: &DatabaseTransaction,
        notification: &notifications::Model,
    ) -> Result<()> {
        // ギルドのロケールを取得
        let locale = self.get_guild_locale(txn, notification.guild_id).await?;

        // メッセージテキストを新しいメッセージサービスで取得
        // DatabaseTransactionはConnectionTraitを実装しているので直接渡せる
        let message_text = self
            .message_service
            .get_message(
                txn,
                &notification.message_text_id,
                HashMap::new(),
                Some(notification.guild_id),
                Some(&locale),
            )
            .await?;

        // チャンネルにメッセージを送信
        let channel_id = ChannelId::new(notification.channel_id as u64);

        let message = CreateMessage::new().content(&message_text);

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
}
