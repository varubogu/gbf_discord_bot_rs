use crate::models::entities::{guild_channels, guilds};
use crate::repository::database::schedule::{NotificationRelEventScheduleRepository, NotificationRepository, ScheduleRepository};
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::services::schedule::{NotificationService, ScheduleCalculator};
use crate::types::{AppState, Result};
use chrono::Utc;
use poise::serenity_prelude::Http;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, TransactionTrait};
use std::collections::HashMap;
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

            // 既存のスケジュールとリレーションをクリア
            debug!("既存のスケジュールを削除します");
            let rel_repo = NotificationRelEventScheduleRepository::new(self.app_state.db().clone());
            rel_repo.delete_all_with_txn(&txn).await?;
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

            // 通知対象のギルドとチャンネルを取得（channel_type別）
            let guild_channels_by_type = self.get_notification_guild_channels_by_type().await?;

            debug!(
                channel_types = guild_channels_by_type.len(),
                "通知対象のギルド・チャンネルを取得しました"
            );

            if guild_channels_by_type.is_empty() {
                warn!("通知対象のギルド・チャンネルが登録されていません");
                return Ok(());
            }

            // スケジュールを計算
            let calculated_schedules = calculator.calculate_schedules(
                event_schedules,
                event_schedule_details,
                guild_channels_by_type,
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

    /// 通知対象のギルド・チャンネル一覧をchannel_type別に取得
    /// 戻り値: HashMap<channel_type, Vec<(guild_id, channel_id)>>
    async fn get_notification_guild_channels_by_type(&self) -> Result<HashMap<i32, Vec<(i64, i64)>>> {
        let guild_channels = guild_channels::Entity::find()
            .all(self.app_state.db())
            .await?;

        let mut channels_by_type: HashMap<i32, Vec<(i64, i64)>> = HashMap::new();

        for gc in guild_channels {
            channels_by_type
                .entry(gc.channel_type)
                .or_insert_with(Vec::new)
                .push((gc.guild_id, gc.channel_id));
        }

        debug!(
            channel_types = channels_by_type.len(),
            total_channels = channels_by_type.values().map(|v| v.len()).sum::<usize>(),
            "channel_type別のギルド・チャンネルを取得しました"
        );

        Ok(channels_by_type)
    }

    /// 計算されたスケジュールをDBに保存
    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        let notification_repo = NotificationRepository::new(self.app_state.db().clone());
        let rel_repo = NotificationRelEventScheduleRepository::new(self.app_state.db().clone());

        debug!(count = schedules.len(), "通知とリレーションを作成します");

        // 各スケジュールに対して通知とリレーションを作成
        for schedule in schedules {
            // 通知を作成
            let notification = notification_repo
                .create_with_txn(
                    txn,
                    schedule.schedule_datetime,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
                .await?;

            // イベントスケジュールとのリレーションを作成
            rel_repo
                .create_with_txn(
                    txn,
                    schedule.event_schedule_id,
                    schedule.event_schedule_detail_id,
                    notification.id,
                )
                .await?;
        }

        debug!("通知とリレーションの作成が完了しました");
        Ok(())
    }
}
