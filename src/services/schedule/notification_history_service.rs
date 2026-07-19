use crate::models::entities::worker::{notifications, scheduled_tasks};
use crate::repository::schedule::{NotificationRepository, ScheduledTaskRepository};
use crate::types::Result;
use chrono::{DateTime, Duration, Utc};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use tracing::{debug, info};

/// 通知とスケジュールタスクの結合結果
#[derive(Debug, Clone)]
pub struct NotificationWithSchedule {
    pub notification: notifications::Model,
    pub schedule_datetime: DateTime<Utc>,
}

/// 通知履歴サービス
/// 送信済み通知の管理を担当
pub struct NotificationHistoryService<N: NotificationRepository, T: ScheduledTaskRepository> {
    notification_repo: N,
    task_repo: T,
}

impl<N: NotificationRepository, T: ScheduledTaskRepository> NotificationHistoryService<N, T> {
    /// 新しいNotificationHistoryServiceインスタンスを作成
    ///
    /// # 引数
    /// * `notification_repo` - 通知リポジトリ
    /// * `task_repo` - スケジュールタスクリポジトリ
    ///
    /// # 戻り値
    /// 新しいNotificationHistoryServiceインスタンス
    pub fn new(notification_repo: N, task_repo: T) -> Self {
        Self {
            notification_repo,
            task_repo,
        }
    }

    /// 過去の通知履歴を取得
    /// 指定した日数分の過去の通知を取得
    pub async fn get_past_notifications(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        days: i64,
    ) -> Result<Vec<NotificationWithSchedule>> {
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
            .find_all_by_datetime_range_with_db(db, from, now)
            .await?;

        // ギルドでフィルタ
        let guild_notifications: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        // scheduled_tasksとJOINしてschedule_datetimeを取得
        // N+1問題を回避するため、バッチでタスクを取得
        let task_ids: Vec<i32> = guild_notifications.iter().map(|n| n.task_id).collect();

        let tasks = self
            .task_repo
            .find_many_by_ids_with_db(db, task_ids)
            .await?;
        let task_map: HashMap<i32, scheduled_tasks::Model> =
            tasks.into_iter().map(|t| (t.id, t)).collect();

        let mut results = Vec::new();
        for notification in guild_notifications {
            if let Some(task) = task_map.get(&notification.task_id) {
                results.push(NotificationWithSchedule {
                    notification,
                    schedule_datetime: task.schedule_datetime,
                });
            }
        }

        info!(count = results.len(), "通知履歴を取得しました");

        Ok(results)
    }

    /// 今後予定されている通知を取得
    pub async fn get_upcoming_notifications(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        days: i64,
    ) -> Result<Vec<NotificationWithSchedule>> {
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
            .find_by_datetime_range_with_db(db, now, to)
            .await?;

        // ギルドでフィルタ
        let guild_notifications: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        // scheduled_tasksとJOINしてschedule_datetimeを取得
        // N+1問題を回避するため、バッチでタスクを取得
        let task_ids: Vec<i32> = guild_notifications.iter().map(|n| n.task_id).collect();

        let tasks = self
            .task_repo
            .find_many_by_ids_with_db(db, task_ids)
            .await?;
        let task_map: HashMap<i32, scheduled_tasks::Model> =
            tasks.into_iter().map(|t| (t.id, t)).collect();

        let mut results = Vec::new();
        for notification in guild_notifications {
            if let Some(task) = task_map.get(&notification.task_id) {
                results.push(NotificationWithSchedule {
                    notification,
                    schedule_datetime: task.schedule_datetime,
                });
            }
        }

        info!(count = results.len(), "今後の通知予定を取得しました");

        Ok(results)
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
            .find_all_by_datetime_range_with_db(db, from, to)
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
