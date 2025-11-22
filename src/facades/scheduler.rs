use crate::models::entities::guilds;
use crate::repository::database::schedule::{NotificationRepository, ScheduleRepository};
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::services::schedule::{NotificationService, ScheduleCalculator};
use crate::types::{AppState, Result};
use chrono::Utc;
use poise::serenity_prelude::Http;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// スケジューラーFacade
/// スケジュール管理の協調とトランザクション管理を担当
pub struct SchedulerFacade {
    app_state: Arc<AppState>,
}

impl SchedulerFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// スケジュールを生成
    /// イベントスケジュールと詳細から通知スケジュールを計算してDBに保存
    pub async fn generate_schedules(&self) -> Result<()> {
        info!("スケジュール生成を開始します");

        let txn = self.app_state.db().begin().await?;

        let result = async {
            let schedule_repo = ScheduleRepository::new(self.app_state.db().clone());
            let notification_repo = NotificationRepository::new(self.app_state.db().clone());
            let calculator = ScheduleCalculator::new();

            // 既存のスケジュールをクリア
            debug!("既存のスケジュールを削除します");
            notification_repo.delete_all_with_txn(&txn).await?;

            // イベントスケジュールと詳細を取得
            let event_schedules = schedule_repo.find_all_event_schedules().await?;
            let event_schedule_details = schedule_repo.find_all_event_schedule_details().await?;

            debug!(
                event_schedules = event_schedules.len(),
                event_details = event_schedule_details.len(),
                "イベントスケジュールを取得しました"
            );

            if event_schedules.is_empty() {
                warn!("イベントスケジュールが登録されていません");
                return Ok(());
            }

            // 通知対象のギルドとチャンネルを取得
            let guild_channels = self.get_notification_guild_channels().await?;

            debug!(
                guild_channels = guild_channels.len(),
                "通知対象のギルド・チャンネルを取得しました"
            );

            if guild_channels.is_empty() {
                warn!("通知対象のギルド・チャンネルが登録されていません");
                return Ok(());
            }

            // スケジュールを計算
            let calculated_schedules = calculator.calculate_schedules(
                event_schedules,
                event_schedule_details,
                guild_channels,
            )?;

            debug!(
                calculated_schedules = calculated_schedules.len(),
                "スケジュールを計算しました"
            );

            // 計算されたスケジュールをDBに保存
            if !calculated_schedules.is_empty() {
                self.save_calculated_schedules(&txn, calculated_schedules)
                    .await?;
            }

            info!("スケジュール生成が完了しました");
            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!("スケジュール生成のトランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "スケジュール生成に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// 通知を実行
    pub async fn execute_notifications(&self, http: Arc<Http>) -> Result<()> {
        debug!("通知実行を開始します");

        let notification_service = NotificationService::new(self.app_state.db().clone(), http);

        notification_service
            .execute_scheduled_notifications()
            .await?;

        debug!("通知実行が完了しました");
        Ok(())
    }

    /// 通知対象のギルド・チャンネル一覧を取得
    async fn get_notification_guild_channels(&self) -> Result<Vec<(i64, i64)>> {
        let guilds = guilds::Entity::find()
            .filter(guilds::Column::RecruitChannelId.is_not_null())
            .all(self.app_state.db())
            .await?;

        let guild_channels: Vec<(i64, i64)> = guilds
            .into_iter()
            .filter_map(|guild| {
                guild
                    .recruit_channel_id
                    .map(|channel_id| (guild.guild_id, channel_id))
            })
            .collect();

        Ok(guild_channels)
    }

    /// 計算されたスケジュールをDBに保存
    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        let notification_repo = NotificationRepository::new(self.app_state.db().clone());

        let notifications_data: Vec<(chrono::DateTime<Utc>, i64, i64, String)> = schedules
            .into_iter()
            .map(|schedule| {
                (
                    schedule.schedule_datetime,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
            })
            .collect();

        notification_repo
            .bulk_create_with_txn(txn, notifications_data)
            .await?;

        Ok(())
    }
}
