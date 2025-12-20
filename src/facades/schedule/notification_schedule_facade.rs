use std::sync::Arc;
use tracing::error;

use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;

use crate::services::schedule::{
    notification_history_service::NotificationHistoryService,
    schedule_query_service::{ScheduleQueryService, ScheduleStats},
};
use crate::types::{AppError, Result, app_state::AppState};

/// 通知スケジュール（未来/履歴）を扱うFacade
/// - Facadeがトランザクション境界を管理
/// - Serviceの協調を行い、Events層からのユースケースを受け付ける
pub struct NotificationScheduleFacade {
    app_state: Arc<AppState>,
}

impl NotificationScheduleFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 未来の通知一覧（ギルド単位）を取得し、表示用に整形した文字列を返す
    pub async fn get_future_notifications_formatted(
        &self,
        guild_id: i64,
        limit: usize,
    ) -> Result<String> {
        let conn = self.app_state.system_db();
        let txn = conn.begin().await?;

        let result = async {
            // 未来の通知は履歴サービスから30日先まで取得
            let history = NotificationHistoryService::new();
            let mut items = history
                .get_upcoming_notifications(conn, guild_id, 30)
                .await?;
            items.sort_by_key(|n| n.schedule_datetime);
            let items = items.into_iter().take(limit).collect::<Vec<_>>();

            let mut s = String::new();
            for (i, n) in items.iter().enumerate() {
                s.push_str(&format!(
                    "{}. <#{}> {}\n",
                    i + 1,
                    n.channel_id,
                    n.schedule_datetime
                ));
            }

            Ok::<_, AppError>(s)
        }
        .await;

        match result {
            Ok(s) => {
                txn.commit().await?;
                Ok(s)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// 通知履歴を取得（from以降）し、統計とともに整形した文字列を返す
    pub async fn get_notification_history_formatted(
        &self,
        guild_id: i64,
        from: DateTime<Utc>,
        limit: usize,
    ) -> Result<(String, ScheduleStats)> {
        let conn = self.app_state.system_db();
        let txn = conn.begin().await?;

        let result = async {
            let history_service = NotificationHistoryService::new();
            // 日数を計算（最低1日）
            let now = Utc::now();
            let days = (now - from).num_days().max(1);
            let items = history_service
                .get_past_notifications(conn, guild_id, days)
                .await?;

            // 統計はScheduleQueryServiceを使用（from/to含む）
            let stats = ScheduleQueryService::new()
                .get_notification_stats(&txn, guild_id, from, now)
                .await?;

            let mut s = String::new();
            for (i, n) in items.iter().take(limit).enumerate() {
                s.push_str(&format!(
                    "{}. <#{}> {} {}\n",
                    i + 1,
                    n.channel_id,
                    n.schedule_datetime,
                    if n.is_sent { "✓" } else { "-" }
                ));
            }

            Ok::<_, AppError>((s, stats))
        }
        .await;

        match result {
            Ok((s, stats)) => {
                txn.commit().await?;
                Ok((s, stats))
            }
            Err(e) => {
                error!(error = %e, guild_id, "通知履歴の取得に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}
