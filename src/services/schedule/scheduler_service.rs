use crate::models::entities::guild_master::guild_channels;
use crate::services::schedule::ScheduleCalculator;
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::types::{AppState, Result};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::models::entities::guild_master::battle_recruitment_schedules;
use crate::models::entities::worker::last_process_times::LastProcessType;
use crate::models::last_process_times::LastProcessTime;
use crate::repository::LastProcessTimeRepository;
use crate::repository::database::last_process_time_repository::SeaOrmLastProcessTimeRepository;
use crate::repository::database::schedule::{
    SeaOrmBattleRecruitmentScheduleRepository, SeaOrmNotificationRelEventScheduleRepository,
    SeaOrmNotificationRepository, SeaOrmScheduleRepository,
};
use crate::repository::schedule::{
    BattleRecruitmentScheduleRepository, NotificationRelEventScheduleRepository,
    NotificationRepository, ScheduleRepository, ScheduledTaskRepository,
};
use sea_orm::DatabaseConnection;

pub struct SchedulerService;

impl Default for SchedulerService {
    fn default() -> Self {
        Self::new()
    }
}

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
        use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
        use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;

        let schedule_repo = SeaOrmScheduleRepository::new();
        let rel_repo = SeaOrmNotificationRelEventScheduleRepository::new();
        let scheduled_task_repo = SeaOrmScheduledTaskRepository::new();
        let calculator = ScheduleCalculator::new();

        // 既存のスケジュールとリレーションをクリア
        debug!("既存のスケジュールを削除します");

        // 1. notification_rel_event_schedulesを削除
        rel_repo.delete_all_with_txn(txn).await?;

        // 2. 通知タイプのscheduled_tasksを削除（CASCADE でnotificationsも削除される）
        let deleted_tasks = scheduled_task_repo
            .delete_all_by_task_type(txn, ScheduledTaskType::Notification.as_i32())
            .await?;

        debug!(
            deleted_tasks = deleted_tasks,
            "通知タイプのscheduled_tasksとnotificationsを削除しました"
        );

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
                .or_default()
                .push((gc.guild_id, gc.channel_id));
        }
        Ok(channels_by_type)
    }

    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
        use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;
        use chrono::Utc;

        let notification_repo = SeaOrmNotificationRepository::new();
        let rel_repo = SeaOrmNotificationRelEventScheduleRepository::new();
        let scheduled_task_repo = SeaOrmScheduledTaskRepository::new();
        let now = Utc::now();

        let mut created_count = 0;
        let mut skipped_count = 0;

        for schedule in schedules {
            if schedule.schedule_datetime < now {
                skipped_count += 1;
                continue;
            }

            // 1. scheduled_taskを作成（task_type=1: Notification）
            let scheduled_task = scheduled_task_repo
                .create(
                    txn,
                    schedule.schedule_datetime,
                    ScheduledTaskType::Notification.as_i32(),
                    Some(schedule.guild_id),
                    Some(schedule.channel_id),
                )
                .await?;

            // 2. notificationを作成（task_idを指定）
            let notification = notification_repo
                .create_with_txn(
                    txn,
                    scheduled_task.id,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
                .await?;

            // 3. notification_relを作成
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

    /// 前回の処理時刻を取得
    ///
    /// # 注意
    /// 通知処理用のlast_process_timesは廃止予定です。
    /// SchedulerManagerを使用してください。
    /// 定期募集処理では引き続き使用されます。
    pub async fn get_last_process_time(
        &self,
        db: &DatabaseConnection,
        process_type: LastProcessType,
    ) -> Result<Option<LastProcessTime>> {
        let last_process_time_repo = SeaOrmLastProcessTimeRepository::new();
        match process_type {
            LastProcessType::Schedule => {
                last_process_time_repo
                    .find_schedule_last_process_time(db)
                    .await
            }
            _ => last_process_time_repo.find_by_type(db, process_type).await,
        }
    }

    /// 処理時刻を更新
    ///
    /// # 注意
    /// 通知処理用のlast_process_timesは廃止予定です。
    /// SchedulerManagerを使用してください。
    /// 定期募集処理では引き続き使用されます。
    pub async fn update_last_process_time(
        &self,
        txn: &DatabaseTransaction,
        process_type: LastProcessType,
        execute_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let last_process_time_repo = SeaOrmLastProcessTimeRepository::new();
        last_process_time_repo
            .upsert_with_txn(txn, process_type, execute_time)
            .await
            .map(|_| ())
    }

    /// 有効な募集スケジュールを曜日情報付きで取得
    pub async fn find_enabled_recruitment_schedules_with_days(
        &self,
        db: &DatabaseConnection,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<crate::models::entities::guild_master::battle_recruitment_schedule_days::Model>,
        )>,
    > {
        let schedule_repo = SeaOrmBattleRecruitmentScheduleRepository::new();
        schedule_repo.find_all_enabled_schedules_with_days(db).await
    }
}
