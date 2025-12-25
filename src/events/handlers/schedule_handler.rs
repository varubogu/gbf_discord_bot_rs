use crate::facades::scheduler::SchedulerFacade;
use crate::types::AppState;
use poise::serenity_prelude::Http;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// スケジュール通知タイマー
/// 10秒間隔で通知を実行
///
/// # 廃止予定
/// このタイマーは廃止予定です。新しいSchedulerManagerを使用してください。
/// SchedulerManagerは自動的に10秒間隔で通知を実行します。
#[deprecated(
    since = "0.1.0",
    note = "Use SchedulerManager instead. This timer will be removed in the future."
)]
pub struct ScheduleNotificationTimer {
    scheduler_facade: Arc<SchedulerFacade>,
    http: Arc<Http>,
    interval: Duration,
}

impl ScheduleNotificationTimer {
    pub fn new(app_state: Arc<AppState>, http: Arc<Http>) -> Self {
        let scheduler_facade = Arc::new(SchedulerFacade::new(app_state));
        Self {
            scheduler_facade,
            http,
            interval: Duration::from_secs(10),
        }
    }

    /// タイマーを起動
    /// 無限ループで10秒ごとに通知を実行
    pub async fn start(self: Arc<Self>) {
        info!(
            "スケジュール通知タイマーを開始します（間隔: {:?}）",
            self.interval
        );

        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            debug!("スケジュール通知を実行します");

            if let Err(e) = self
                .scheduler_facade
                .execute_notifications(self.http.clone())
                .await
            {
                error!(error = %e, "スケジュール通知の実行に失敗しました");
            }
        }
    }
}
