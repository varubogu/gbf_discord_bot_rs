use crate::gateway::DiscordGateway;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::schedule::{
    BattleRecruitmentScheduleRepository, ScheduledTaskRecurringRecruitmentRepository,
    ScheduledTaskRepository,
};
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::schedule::{CalculatedRecruitmentTime, RecruitmentScheduleService};
use crate::types::{AppError, Result};
use chrono::{Duration, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 定期募集タスク実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum RecurringRecruitmentExecutionResult {
    /// 実行成功（マルチ募集を作成した）
    Success { next_task_id: i32 },
    /// スケジュールが見つからない（削除済み）
    ScheduleNotFound { schedule_id: i32 },
    /// スケジュールが無効化されている
    ScheduleDisabled { schedule_id: i32 },
    /// 次回実行日時が見つからない
    NextExecutionNotFound { schedule_id: i32 },
}

/// 定期募集タスク実行サービス
///
/// 設計: scheduled_task_recurring_recruitmentsからスケジュール情報を取得し、
/// マルチ募集を作成して、次回実行タスクをscheduled_tasksに登録する
pub struct RecurringRecruitmentTaskExecutor<
    ST,
    RR,
    SR,
    GC,
    Q,
    BS,
    A,
    QR,
    GE,
    SD,
    GM,
    MT,
    NMN,
    NMR,
    NMS,
    DR,
    TR,
    TDR,
    GS,
    BR,
> where
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
    GC: crate::repository::GuildChannelRepository,
    Q: crate::repository::QuestRepository,
    BS: crate::repository::BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: crate::repository::BattleRecruitmentsRepository,
{
    task_repo: ST,
    recurring_repo: RR,
    schedule_repo: SR,
    schedule_service: Arc<RecruitmentScheduleService>,
    recruitment_creation_service: Arc<
        RecruitmentCreationService<
            GC,
            Q,
            BS,
            A,
            QR,
            GE,
            SD,
            GM,
            MT,
            NMN,
            NMR,
            NMS,
            DR,
            TR,
            TDR,
            GS,
            BR,
        >,
    >,
}

impl<ST, RR, SR, GC, Q, BS, A, QR, GE, SD, GM, MT, NMN, NMR, NMS, DR, TR, TDR, GS, BR>
    RecurringRecruitmentTaskExecutor<
        ST,
        RR,
        SR,
        GC,
        Q,
        BS,
        A,
        QR,
        GE,
        SD,
        GM,
        MT,
        NMN,
        NMR,
        NMS,
        DR,
        TR,
        TDR,
        GS,
        BR,
    >
where
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
    GC: crate::repository::GuildChannelRepository,
    Q: crate::repository::QuestRepository,
    BS: crate::repository::BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: crate::repository::BattleRecruitmentsRepository,
{
    pub fn new(
        task_repo: ST,
        recurring_repo: RR,
        schedule_repo: SR,
        schedule_service: Arc<RecruitmentScheduleService>,
        recruitment_creation_service: Arc<
            RecruitmentCreationService<
                GC,
                Q,
                BS,
                A,
                QR,
                GE,
                SD,
                GM,
                MT,
                NMN,
                NMR,
                NMS,
                DR,
                TR,
                TDR,
                GS,
                BR,
            >,
        >,
    ) -> Self {
        Self {
            task_repo,
            recurring_repo,
            schedule_repo,
            schedule_service,
            recruitment_creation_service,
        }
    }

    /// 定期募集タスクを実行する
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `db_conn` - データベース接続
    /// * `gateway` - Discord Gateway
    /// * `task_id` - 実行対象のタスクID
    ///
    /// # 戻り値
    /// * `Ok(RecurringRecruitmentExecutionResult)` - 実行結果
    ///
    /// # エラー
    /// * タスクが見つからない場合
    /// * DB操作でエラーが発生した場合
    pub async fn execute<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &G,
        task_id: i32,
    ) -> Result<RecurringRecruitmentExecutionResult> {
        info!(task_id, "定期募集タスク実行開始");

        // タスクが削除されていないか、既に実行済みでないかを確認
        let _task = match self.task_repo.find_by_id(txn, task_id).await? {
            Some(task) if !task.is_executed => task,
            Some(_) => {
                warn!(task_id, "タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("Task {task_id} is already executed"),
                });
            }
            None => {
                warn!(task_id, "タスクが見つかりません");
                return Err(AppError::Business {
                    message: format!("Task {task_id} not found"),
                });
            }
        };

        // 定期募集情報を取得
        let recurring = match self
            .recurring_repo
            .find_by_scheduled_task_id(txn, task_id)
            .await?
        {
            Some(r) => r,
            None => {
                error!(task_id, "定期募集情報が見つかりません");
                return Err(AppError::Business {
                    message: format!("Recurring recruitment info not found for task {task_id}"),
                });
            }
        };

        let schedule_id = recurring.recruitment_schedule_id;

        // スケジュール情報を取得
        let schedule_and_days = self.schedule_repo.find_by_id(txn, schedule_id).await?;
        let (schedule, days) = match schedule_and_days {
            Some(s) => s,
            None => {
                warn!(
                    task_id,
                    schedule_id, "スケジュールが見つかりません（削除済み）"
                );
                // タスクを実行済みにマーク
                self.task_repo.mark_as_executed(txn, task_id).await?;
                return Ok(RecurringRecruitmentExecutionResult::ScheduleNotFound { schedule_id });
            }
        };

        // スケジュールが有効かチェック
        if !schedule.is_enabled {
            info!(task_id, schedule_id, "スケジュールは無効化されています");
            // タスクを実行済みにマーク
            self.task_repo.mark_as_executed(txn, task_id).await?;
            return Ok(RecurringRecruitmentExecutionResult::ScheduleDisabled { schedule_id });
        }

        // マルチ募集を作成（CalculatedRecruitmentTimeを構築）
        info!(
            task_id,
            schedule_id,
            quest_id = schedule.quest_id,
            "マルチ募集を作成します"
        );

        // CalculatedRecruitmentTimeを作成
        let calculated_time = CalculatedRecruitmentTime {
            schedule_id: schedule.id,
            guild_id: schedule.guild_id,
            channel_id: schedule.channel_id,
            quest_id: schedule.quest_id,
            battle_style_id: schedule.battle_style_id,
            quest_start_at: chrono::Utc::now(), // 実行時点で即座に開始（実際の開始時刻はscheduleから取得されるべきだが、既存実装に合わせる）
            recruit_start_at: chrono::Utc::now(),
            max_participants: schedule.max_participants,
            note: schedule.note.clone(),
        };

        self.recruitment_creation_service
            .create_recruitment_from_schedule(txn, db_conn, gateway, &calculated_time)
            .await?;

        info!(task_id, schedule_id, "マルチ募集を作成しました");

        // 次回実行日時を計算してscheduled_tasksに登録
        let next_task_id = self
            .create_next_scheduled_task(txn, &schedule, &days)
            .await?;

        // 現在のタスクを実行済みにマーク
        self.task_repo.mark_as_executed(txn, task_id).await?;

        info!(task_id, schedule_id, next_task_id, "定期募集タスク実行完了");

        Ok(RecurringRecruitmentExecutionResult::Success { next_task_id })
    }

    /// 次回実行タスクをscheduled_tasksに登録
    ///
    /// 現在時刻から未来の次回実行日時を計算し、scheduled_tasksとscheduled_task_recurring_recruitmentsに登録
    /// 過去日時の場合は未来日時が見つかるまで繰り返し計算
    async fn create_next_scheduled_task(
        &self,
        txn: &DatabaseTransaction,
        schedule: &crate::models::entities::guild_master::battle_recruitment_schedules::Model,
        days: &[crate::models::entities::guild_master::battle_recruitment_schedule_days::Model],
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

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
