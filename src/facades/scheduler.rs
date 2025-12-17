use crate::models::entities::last_process_times::LastProcessType;
use crate::services::schedule::RecruitmentScheduleService;
use crate::services::schedule::{
    notification_service::NotificationService, scheduler_service::SchedulerService,
};
use crate::types::{AppState, Result};
use chrono::Utc;
use poise::serenity_prelude::Http;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{debug, error, info};

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

        // スケジュール生成はSystemロールを使用（全ギルド対象）
        let txn = self.app_state.system_db().begin().await?;

        let result = async {
            let service = SchedulerService::new();
            service
                .generate_and_persist_schedules(&txn, &self.app_state)
                .await?;
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
    /// last_process_timesから前回実行時刻を取得し、その時刻から現在までの通知を実行
    /// 各通知はis_sentフラグで管理されるため、last_process_timesの更新は不要
    pub async fn execute_notifications(&self, http: Arc<Http>) -> Result<()> {
        debug!("通知実行を開始します");

        let now = Utc::now();
        let scheduler_service = SchedulerService::new();

        // 前回のスケジュール実行時刻を取得
        let last_process_time = scheduler_service
            .get_last_process_time(self.app_state.system_db(), LastProcessType::Schedule)
            .await?;

        let last_execute_time = last_process_time.and_then(|lpt| lpt.execute_time);

        debug!(
            last_execute_time = ?last_execute_time,
            "前回のスケジュール実行時刻を取得しました"
        );

        // 通知を実行（Facadeでトランザクション管理）
        let txn = self.app_state.system_db().begin().await?;
        let notification_service = NotificationService::new(http.clone());
        let exec_result = notification_service
            .execute_scheduled_notifications(&txn, last_execute_time)
            .await;

        match exec_result {
            Ok(_) => {
                txn.commit().await?;
            }
            Err(e) => {
                error!(error = %e, "通知実行に失敗しました");
                txn.rollback().await?;
                return Err(e);
            }
        }

        // 定期募集を実行
        self.execute_recruitment_schedules(http.clone()).await?;

        // last_process_timesを更新（次回実行時の検索範囲を決定するため）
        let txn = self.app_state.system_db().begin().await?;

        let result = scheduler_service
            .update_last_process_time(&txn, LastProcessType::Schedule, now)
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("last_process_timesを更新しました");
            }
            Err(e) => {
                error!(error = %e, "last_process_timesの更新に失敗しました");
                txn.rollback().await?;
                return Err(e);
            }
        }

        debug!("通知実行が完了しました");
        Ok(())
    }

    /// 定期募集を実行
    /// 有効なスケジュールから募集を作成し、battle_recruitmentsに登録
    pub async fn execute_recruitment_schedules(&self, _http: Arc<Http>) -> Result<()> {
        debug!("定期募集実行を開始します");

        let now = Utc::now();
        let scheduler_service = SchedulerService::new();

        // 前回の定期募集実行時刻を取得
        let last_process_time = scheduler_service
            .get_last_process_time(
                self.app_state.system_db(),
                LastProcessType::BattleRecruitmentSchedule,
            )
            .await?;

        let last_execute_time = last_process_time.and_then(|lpt| lpt.execute_time);

        let from = last_execute_time.unwrap_or(now);

        debug!(
            last_execute_time = ?last_execute_time,
            from = %from,
            to = %now,
            "前回の定期募集実行時刻を取得しました"
        );

        // 有効な全スケジュールと曜日情報を取得
        let schedules = scheduler_service
            .find_enabled_recruitment_schedules_with_days(self.app_state.system_db())
            .await?;

        debug!(
            schedule_count = schedules.len(),
            "有効なスケジュールを取得しました"
        );

        if schedules.is_empty() {
            debug!("実行対象のスケジュールはありません");

            // last_process_timesを更新
            let txn = self.app_state.system_db().begin().await?;
            let result = scheduler_service
                .update_last_process_time(&txn, LastProcessType::BattleRecruitmentSchedule, now)
                .await;

            match result {
                Ok(_) => {
                    txn.commit().await?;
                    debug!("last_process_timesを更新しました");
                }
                Err(e) => {
                    error!(error = %e, "last_process_timesの更新に失敗しました");
                    txn.rollback().await?;
                    return Err(e);
                }
            }

            return Ok(());
        }

        // 各スケジュールについて募集日時を計算
        // DBの値は既にUTCなので、タイムゾーン取得・変換は不要
        let recruitment_service = RecruitmentScheduleService::new();
        let mut all_calculated_times = Vec::new();

        for (schedule, days) in &schedules {
            let calculated_times =
                recruitment_service.calculate_next_recruitment_times(schedule, days, from, now)?;

            debug!(
                schedule_id = schedule.id,
                calculated_count = calculated_times.len(),
                "募集日時を計算しました（UTC基準）"
            );

            all_calculated_times.extend(calculated_times);
        }

        info!(
            total_calculated = all_calculated_times.len(),
            "全スケジュールの募集日時計算が完了しました"
        );

        // 募集作成
        let mut created_count = 0;
        let mut skipped_count = 0;

        for calculated_time in all_calculated_times {
            // 募集作成処理を実行
            let create_result = self
                .create_recruitment_from_schedule(&_http, &calculated_time)
                .await;

            match create_result {
                Ok(_) => {
                    info!(
                        schedule_id = calculated_time.schedule_id,
                        quest_start_at = %calculated_time.quest_start_at,
                        "定期募集を作成しました"
                    );
                    created_count += 1;
                }
                Err(e) => {
                    error!(
                        error = %e,
                        schedule_id = calculated_time.schedule_id,
                        quest_start_at = %calculated_time.quest_start_at,
                        "定期募集の作成に失敗しました"
                    );
                    // エラーをログに記録するが、処理は継続する
                    skipped_count += 1;
                }
            }
        }

        info!(
            created = created_count,
            skipped = skipped_count,
            "定期募集の作成が完了しました"
        );

        // last_process_timesを更新
        let txn = self.app_state.system_db().begin().await?;

        let result = scheduler_service
            .update_last_process_time(&txn, LastProcessType::BattleRecruitmentSchedule, now)
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("last_process_timesを更新しました");
            }
            Err(e) => {
                error!(error = %e, "last_process_timesの更新に失敗しました");
                txn.rollback().await?;
                return Err(e);
            }
        }

        debug!("定期募集実行が完了しました");
        Ok(())
    }

    /// スケジュールから募集を作成
    /// 定期募集スケジュールに基づいて実際の募集を作成
    async fn create_recruitment_from_schedule(
        &self,
        http: &Arc<Http>,
        calculated_time: &crate::services::schedule::CalculatedRecruitmentTime,
    ) -> Result<()> {
        debug!(
            schedule_id = calculated_time.schedule_id,
            quest_id = calculated_time.quest_id,
            "スケジュールから募集を作成します"
        );

        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        let recruitment_creation_service =
            crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService::new(
            );

        let result = recruitment_creation_service
            .create_recruitment_from_schedule(&txn, conn, http, calculated_time)
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("募集作成トランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "募集作成トランザクションをロールバックしました");
                Err(e)
            }
        }
    }
}
