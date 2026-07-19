use crate::models::entities::guild_master::{
    battle_recruitment_schedule_days, battle_recruitment_schedules,
};
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::models::entities::worker::{scheduled_task_recurring_recruitments, scheduled_tasks};
use crate::repository::schedule::{
    BattleRecruitmentScheduleRepository, ScheduledTaskRecurringRecruitmentRepository,
    ScheduledTaskRepository,
};
use crate::services::schedule::RecruitmentScheduleService;
use crate::types::{AppError, Result};
use chrono::{Duration, Utc};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{debug, info};

/// 定期募集タスクディスパッチが必要とする scheduled_tasks /
/// scheduled_task_recurring_recruitments / battle_recruitment_schedules への
/// 直接アクセスと、次回実行タスクの登録ロジックを集約する薄いサービス。
///
/// facade層がrepositoryを直接呼ばずに済むよう、ディスパッチ処理専用の窓口として存在する。
pub struct RecurringRecruitmentDispatchSupportService<ST, RR, SR>
where
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
{
    task_repo: ST,
    recurring_repo: RR,
    schedule_repo: SR,
    schedule_service: Arc<RecruitmentScheduleService>,
}

impl<ST, RR, SR> RecurringRecruitmentDispatchSupportService<ST, RR, SR>
where
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
{
    pub fn new(
        task_repo: ST,
        recurring_repo: RR,
        schedule_repo: SR,
        schedule_service: Arc<RecruitmentScheduleService>,
    ) -> Self {
        Self {
            task_repo,
            recurring_repo,
            schedule_repo,
            schedule_service,
        }
    }

    pub async fn find_task(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_tasks::Model>> {
        self.task_repo.find_by_id(txn, task_id).await
    }

    pub async fn find_recurring_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_task_recurring_recruitments::Model>> {
        self.recurring_repo
            .find_by_scheduled_task_id(txn, task_id)
            .await
    }

    #[allow(clippy::type_complexity)]
    pub async fn find_schedule(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<
        Option<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    > {
        self.schedule_repo.find_by_id(txn, schedule_id).await
    }

    pub async fn mark_succeeded_with_warning(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<()> {
        self.task_repo
            .mark_as_succeeded_with_warning(txn, task_id)
            .await?;
        Ok(())
    }

    pub async fn mark_succeeded(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
        self.task_repo.mark_as_succeeded(txn, task_id).await?;
        Ok(())
    }

    /// 次回実行タスクをscheduled_tasksに登録
    ///
    /// 現在時刻から未来の次回実行日時を計算し、scheduled_tasksとscheduled_task_recurring_recruitmentsに登録する。
    /// 過去日時の場合は未来日時が見つかるまで繰り返し計算する。
    pub async fn register_next_scheduled_task(
        &self,
        txn: &DatabaseTransaction,
        schedule: &battle_recruitment_schedules::Model,
        days: &[battle_recruitment_schedule_days::Model],
    ) -> Result<i32> {
        debug!(
            schedule_id = schedule.id,
            "次回実行タスクの作成を開始します"
        );

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
            let next_times = self.schedule_service.calculate_next_recruitment_times(
                schedule,
                days,
                search_from,
                search_to,
            )?;

            // 最初に見つかった未来の募集開始日時を使用
            if let Some(next_time) = next_times.first()
                && next_time.recruit_start_at > now
            {
                // 未来日時が見つかった場合、scheduled_tasksに登録
                let task = self
                    .task_repo
                    .create(
                        txn,
                        next_time.recruit_start_at,
                        ScheduledTaskType::RecurringRecruitment as i32,
                        Some(next_time.guild_id),
                        Some(next_time.channel_id),
                    )
                    .await?;

                // scheduled_task_recurring_recruitmentsに関連付けを登録
                self.recurring_repo
                    .create(txn, task.id, schedule.id)
                    .await?;

                info!(
                    schedule_id = schedule.id,
                    task_id = task.id,
                    recruit_start_at = %next_time.recruit_start_at,
                    "次回実行タスクを登録しました"
                );

                return Ok(task.id);
            }

            // 次の検索範囲に進む
            search_from = search_to;

            // 無限ループ防止：最大検索日数を超えたらエラー
            if (search_from - now).num_days() > max_search_days {
                return Err(AppError::Business {
                    message: format!(
                        "次回実行日時が{max_search_days}日以内に見つかりませんでした。スケジュール設定を確認してください。"
                    ),
                });
            }
        }
    }
}
