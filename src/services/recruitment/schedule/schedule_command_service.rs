use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::database::schedule::{
    BattleRecruitmentScheduleRepository, ScheduledTaskRecurringRecruitmentRepository,
    ScheduledTaskRepository,
};
use crate::services::schedule::RecruitmentScheduleService;
use crate::types::Result;
use chrono::{Duration, Utc};
use sea_orm::DatabaseTransaction;
use tracing::{debug, info};

/// スケジュール操作サービス（削除・有効/無効切替）
///
/// Facade層から呼び出され、Repository層への具体的なアクセスを集約する。
/// RLS設定はFacade層で行われるため、このService層では行わない。
pub struct ScheduleCommandService;

impl ScheduleCommandService {
    pub fn new() -> Self {
        Self
    }

    /// スケジュールを削除
    ///
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn delete_schedule(&self, txn: &DatabaseTransaction, schedule_id: i32) -> Result<()> {
        // 1. スケジュールを無効化（未実行タスクとリレーションを削除）
        self.disable_schedule(txn, schedule_id).await?;

        // 2. battle_recruitment_schedules を削除
        let schedule_repo = BattleRecruitmentScheduleRepository::new();
        schedule_repo.delete_with_txn(txn, schedule_id).await?;

        Ok(())
    }

    /// スケジュールの現在の有効/無効状態を取得
    ///
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn get_schedule_enabled_status(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<bool> {
        let repo = BattleRecruitmentScheduleRepository::new();

        let (schedule, _) = repo.find_by_id(txn, schedule_id).await?.ok_or_else(|| {
            crate::types::AppError::NotFound(format!(
                "スケジュールID {schedule_id} が見つかりません"
            ))
        })?;

        Ok(schedule.is_enabled)
    }

    /// スケジュールを有効化
    ///
    /// 次回実行タスクをscheduled_tasksに登録する
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn enable_schedule(&self, txn: &DatabaseTransaction, schedule_id: i32) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();

        // スケジュールを取得
        let (schedule, days) = repo.find_by_id(txn, schedule_id).await?.ok_or_else(|| {
            crate::types::AppError::NotFound(format!(
                "スケジュールID {schedule_id} が見つかりません"
            ))
        })?;

        // 有効化
        repo.toggle_enabled_with_txn(txn, schedule_id, true).await?;

        // 次回実行タスクを登録
        self.create_next_scheduled_task(txn, &schedule, &days)
            .await?;

        Ok(())
    }

    /// スケジュールを無効化（一時停止）
    ///
    /// 未実行のscheduled_tasksとscheduled_task_recurring_recruitmentsを削除する
    /// 既に実行済み（募集開始済み）のタスクは削除しない（募集は独立して存在）
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn disable_schedule(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();

        // スケジュールが存在することを確認
        repo.find_by_id(txn, schedule_id).await?.ok_or_else(|| {
            crate::types::AppError::NotFound(format!(
                "スケジュールID {schedule_id} が見つかりません"
            ))
        })?;

        let recurring_task_repo = ScheduledTaskRecurringRecruitmentRepository::new();

        // 未実行の scheduled_tasks を削除
        recurring_task_repo
            .delete_pending_tasks_by_recruitment_schedule_id(txn, schedule_id)
            .await?;

        // scheduled_task_recurring_recruitments を削除
        recurring_task_repo
            .delete_by_recruitment_schedule_id(txn, schedule_id)
            .await?;

        // 無効化
        repo.toggle_enabled_with_txn(txn, schedule_id, false)
            .await?;

        Ok(())
    }

    /// 次回実行タスクをscheduled_tasksに登録
    ///
    /// 現在時刻から未来の次回実行日時を計算し、scheduled_tasksとscheduled_task_recurring_recruitmentsに登録
    /// 最も近い募集開始日時のスケジュールを1つだけ作成する
    async fn create_next_scheduled_task(
        &self,
        txn: &DatabaseTransaction,
        schedule: &crate::models::entities::guild_master::battle_recruitment_schedules::Model,
        days: &[crate::models::entities::guild_master::battle_recruitment_schedule_days::Model],
    ) -> Result<()> {
        debug!(
            schedule_id = schedule.id,
            "次回実行タスクの作成を開始します"
        );

        let schedule_service = RecruitmentScheduleService::new();
        let now = Utc::now();
        let mut search_from = now;
        let max_search_days = 365; // 最大1年先まで検索

        // 未来の次回実行日時が見つかるまでループ
        loop {
            let search_to = search_from + Duration::days(7);

            debug!(
                schedule_id = schedule.id,
                search_from = %search_from,
                search_to = %search_to,
                "次回実行日時を計算します"
            );

            // 次回募集日時を計算
            let next_times = schedule_service.calculate_next_recruitment_times(
                schedule,
                days,
                search_from,
                search_to,
            )?;

            // 最初に見つかった未来の募集開始日時を使用
            if let Some(next_time) = next_times.first() {
                if next_time.recruit_start_at > now {
                    // 未来日時が見つかった場合、scheduled_tasksに登録
                    let task_repo = ScheduledTaskRepository::new();
                    let task = task_repo
                        .create(
                            txn,
                            next_time.recruit_start_at,
                            ScheduledTaskType::RecurringRecruitment as i32,
                            Some(next_time.guild_id),
                            Some(next_time.channel_id),
                        )
                        .await?;

                    // scheduled_task_recurring_recruitmentsに関連付けを登録
                    let recurring_repo = ScheduledTaskRecurringRecruitmentRepository::new();
                    recurring_repo.create(txn, task.id, schedule.id).await?;

                    info!(
                        schedule_id = schedule.id,
                        task_id = task.id,
                        recruit_start_at = %next_time.recruit_start_at,
                        "次回実行タスクを登録しました"
                    );

                    return Ok(());
                }
            }

            // 次の検索範囲に進む
            search_from = search_to;

            // 無限ループ防止：最大検索日数を超えたらエラー
            if (search_from - now).num_days() > max_search_days {
                return Err(crate::types::AppError::Business {
                    message: format!(
                        "次回実行日時が{max_search_days}日以内に見つかりませんでした。スケジュール設定を確認してください。"
                    ),
                });
            }
        }
    }
}

impl Default for ScheduleCommandService {
    fn default() -> Self {
        Self::new()
    }
}
