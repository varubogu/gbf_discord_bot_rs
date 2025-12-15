use crate::models::entities::guild_channels;
use crate::services::schedule::ScheduleCalculator;
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::types::{AppState, Result};
use chrono::Utc;
use sea_orm::{DatabaseTransaction, EntityTrait};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::repository::database::schedule::{
    NotificationRelEventScheduleRepository, NotificationRepository, ScheduleRepository,
};

pub struct SchedulerService;

impl SchedulerService {
    pub fn new() -> Self {
        Self
    }

    /// イベントスケジュールから通知スケジュールを計算し保存する
    /// - トランザクション境界はFacadeが管理
    pub async fn generate_and_persist_schedules(
        &self,
        txn: &DatabaseTransaction,
        app_state: &AppState,
    ) -> Result<()> {
        let schedule_repo = ScheduleRepository::new();
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelEventScheduleRepository::new();
        let calculator = ScheduleCalculator::new();

        // 既存のスケジュールとリレーションをクリア
        debug!("既存のスケジュールを削除します");
        rel_repo.delete_all_with_txn(txn).await?;
        notification_repo.delete_all_with_txn(txn).await?;

        // イベントスケジュールと詳細を取得
        let event_schedules = schedule_repo
            .find_all_event_schedules(app_state.system_db())
            .await?;
        let event_schedule_details = schedule_repo
            .find_all_event_schedule_details(app_state.system_db())
            .await?;

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
        let guild_channels_by_type = self
            .get_notification_guild_channels_by_type(app_state)
            .await?;

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
            self.save_calculated_schedules(txn, calculated_schedules)
                .await?;
        }

        info!("スケジュール生成が完了しました");
        Ok(())
    }

    async fn get_notification_guild_channels_by_type(
        &self,
        app_state: &AppState,
    ) -> Result<HashMap<i32, Vec<(i64, i64)>>> {
        let guild_channels = <guild_channels::Entity as sea_orm::EntityTrait>::find()
            .all(app_state.system_db())
            .await?;

        let mut channels_by_type: HashMap<i32, Vec<(i64, i64)>> = HashMap::new();

        for gc in guild_channels {
            channels_by_type
                .entry(gc.channel_type)
                .or_insert_with(Vec::new)
                .push((gc.guild_id, gc.channel_id));
        }
        Ok(channels_by_type)
    }

    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        use chrono::Utc;
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelEventScheduleRepository::new();
        let now = Utc::now();

        let mut created_count = 0;
        let mut skipped_count = 0;

        for schedule in schedules {
            if schedule.schedule_datetime < now {
                skipped_count += 1;
                continue;
            }

            let notification = notification_repo
                .create_with_txn(
                    txn,
                    schedule.schedule_datetime,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
                .await?;

            rel_repo
                .create_with_txn(
                    txn,
                    schedule.event_schedule_id,
                    schedule.event_schedule_detail_id,
                    notification.id,
                )
                .await?;

            created_count += 1;
        }

        debug!(
            created = created_count,
            skipped = skipped_count,
            "通知とリレーションの作成が完了しました"
        );
        Ok(())
    }
}
