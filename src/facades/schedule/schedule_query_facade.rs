use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::schedule::{ScheduleListItem, ScheduleQueryService, ScheduleStats};
use crate::services::timezone_service::TimezoneService;
use crate::types::app_state::AppState;
use crate::types::{AppError, Result};
use chrono::{Duration, Utc};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// スケジュールクエリファサード
///
/// スケジュール一覧取得・統計取得のユースケースを管理するファサード。
/// トランザクション境界の管理を担当。
pub struct ScheduleQueryFacade {
    app_state: Arc<AppState>,
}

impl ScheduleQueryFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 通知統計を取得
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `days`: 統計期間（日数）
    ///
    /// # 戻り値
    /// スケジュール統計
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    pub async fn get_stats(&self, guild_id: i64, days: i64) -> Result<ScheduleStats> {
        info!(guild_id = guild_id, days = days, "通知統計取得を開始します");

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let now = Utc::now();
            let from = now - Duration::days(days);

            // 通知統計取得
            let query_service = ScheduleQueryService::new();
            let stats = query_service
                .get_notification_stats(&txn, guild_id, from, now)
                .await?;

            Ok::<_, AppError>(stats)
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(stats) => {
                txn.commit().await?;
                info!(
                    guild_id = guild_id,
                    total_count = stats.total_count,
                    "通知統計取得に成功しました"
                );
                Ok(stats)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "通知統計取得に失敗しました"
                );
                Err(e)
            }
        }
    }
}
