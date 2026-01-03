use crate::models::entities::worker::notifications;
use crate::repository::database::schedule::SeaOrmNotificationRepository;
use crate::repository::schedule::NotificationRepository;
use crate::types::Result;
use chrono::{DateTime, Duration, Utc};
use sea_orm::DatabaseConnection;
use tracing::{debug, info};

/// 通知履歴サービス
/// 送信済み通知の管理を担当
pub struct NotificationHistoryService {
    notification_repo: SeaOrmNotificationRepository,
}

impl Default for NotificationHistoryService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationHistoryService {
    pub fn new() -> Self {
        let notification_repo = SeaOrmNotificationRepository::new();
        Self { notification_repo }
    }

    /// 過去の通知履歴を取得
    /// 指定した日数分の過去の通知を取得
    pub async fn get_past_notifications(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        days: i64,
    ) -> Result<Vec<notifications::Model>> {
        let now = Utc::now();
        let from = now - Duration::days(days);

        debug!(
            guild_id = guild_id,
            from = %from,
            to = %now,
            "過去の通知履歴を取得します"
        );

        let notifications = self
            .notification_repo
            .find_by_datetime_range(db, from, now)
            .await?;

        // ギルドでフィルタ
        let guild_notifications: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        info!(count = guild_notifications.len(), "通知履歴を取得しました");

        Ok(guild_notifications)
    }

    /// 今後予定されている通知を取得
    pub async fn get_upcoming_notifications(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        days: i64,
    ) -> Result<Vec<notifications::Model>> {
        let now = Utc::now();
        let to = now + Duration::days(days);

        debug!(
            guild_id = guild_id,
            from = %now,
            to = %to,
            "今後の通知予定を取得します"
        );

        let notifications = self
            .notification_repo
            .find_by_datetime_range(db, now, to)
            .await?;

        // ギルドでフィルタ
        let guild_notifications: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        info!(
            count = guild_notifications.len(),
            "今後の通知予定を取得しました"
        );

        Ok(guild_notifications)
    }

    /// 特定期間の通知統計を取得
    pub async fn get_notification_stats(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<NotificationStats> {
        debug!(
            guild_id = guild_id,
            from = %from,
            to = %to,
            "通知統計を取得します"
        );

        let notifications = self
            .notification_repo
            .find_by_datetime_range(db, from, to)
            .await?;

        let guild_notifications: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        let total_count = guild_notifications.len();

        // メッセージタイプ別の集計
        let mut message_type_counts = std::collections::HashMap::new();
        for notification in &guild_notifications {
            *message_type_counts
                .entry(notification.message_text_id.clone())
                .or_insert(0) += 1;
        }

        Ok(NotificationStats {
            total_count,
            message_type_counts,
        })
    }
}

/// 通知統計
#[derive(Debug, Clone)]
pub struct NotificationStats {
    pub total_count: usize,
    pub message_type_counts: std::collections::HashMap<String, usize>,
}
