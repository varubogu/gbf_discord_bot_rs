use super::shared_presentation::build_v2_recruitment_embed_and_components;
use super::task_timing::should_skip_recruitment_creation;
use crate::gateway::DiscordGateway;
use crate::models::entities::guild_master::{
    battle_recruitment_schedule_days, battle_recruitment_schedules,
};
use crate::models::entities::worker::{scheduled_task_recurring_recruitments, scheduled_tasks};
use crate::repository::schedule::{
    BattleRecruitmentDismissalRepository, BattleRecruitmentScheduleDismissalRepository,
    BattleRecruitmentScheduleRepository, NotificationRelBattleRecruitmentRepository,
    NotificationRepository, ScheduledTaskDismissalRepository,
    ScheduledTaskRecurringRecruitmentRepository, ScheduledTaskRepository,
};
use crate::repository::{
    AllRecruitmentNotificationRolesRepository, BattleRecruitmentsRepository, BattleStyleRepository,
    GuildChannelRepository, GuildEnvironmentRepository, GuildMessageTextRepository,
    GuildSettingsRepository, MessageTextRepository, QuestRecruitmentNotificationRolesRepository,
    QuestRepository,
};
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::schedule::recurring_recruitment_dispatch_support_service::RecurringRecruitmentDispatchSupportService;
use crate::services::schedule::{
    CalculatedRecruitmentTime, RecruitmentMessageDeletionScheduler, RecruitmentScheduleService,
};
use crate::types::discord::{DiscordChannelId, MessageContent};
use crate::types::{AppError, Result};
use chrono::Utc;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use tracing::{debug, error, info, warn};

/// 定期募集タスク実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum RecurringRecruitmentExecutionResult {
    /// 実行成功（マルチ募集を作成した）
    Success { next_task_id: i32 },
    /// 過去タスクをスキップ後、現在募集可能な回を即時実行した
    RecoveredCurrentWindow { schedule_id: i32, next_task_id: i32 },
    /// 出発時刻が過去のため募集作成をスキップし、次回実行のみ登録した
    SkippedPastDeparture { schedule_id: i32, next_task_id: i32 },
    /// スケジュールが見つからない（削除済み）
    ScheduleNotFound { schedule_id: i32 },
    /// スケジュールが無効化されている
    ScheduleDisabled { schedule_id: i32 },
    /// 次回実行日時が見つからない
    NextExecutionNotFound { schedule_id: i32 },
}

/// タスクと定期募集情報を取得し、実行可能な状態であることを検証する
async fn find_pending_recurring_task<ST, RR, SR>(
    txn: &DatabaseTransaction,
    dispatch_support: &RecurringRecruitmentDispatchSupportService<ST, RR, SR>,
    task_id: i32,
) -> Result<(
    scheduled_tasks::Model,
    scheduled_task_recurring_recruitments::Model,
)>
where
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
{
    // タスクが削除されていないか、既に実行済みでないかを確認
    let task = match dispatch_support.find_task(txn, task_id).await? {
        Some(task) if task.execution_status.is_pending() => task,
        Some(_) => {
            warn!(task_id, "タスクは既に実行済みです");
            return Err(AppError::Business {
                message: format!("Task {task_id} is not pending"),
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
    let recurring = match dispatch_support
        .find_recurring_by_task_id(txn, task_id)
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

    Ok((task, recurring))
}

/// 定期募集タスクを実行する
///
/// 元は`RecurringRecruitmentTaskExecutor`（service層）にあったロジック。
/// Facadeが複数serviceを合成してユースケースを実行する責務に合わせてここへ統合した。
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_recurring_recruitment_dispatch<
    G,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    dispatch_support: &RecurringRecruitmentDispatchSupportService<ST, RR, SR>,
    schedule_service: &RecruitmentScheduleService,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    task_id: i32,
) -> Result<RecurringRecruitmentExecutionResult>
where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    info!(task_id, "定期募集タスク実行開始");

    let (task, recurring) = find_pending_recurring_task(txn, dispatch_support, task_id).await?;

    let schedule_id = recurring.recruitment_schedule_id;

    // スケジュール情報を取得
    let schedule_and_days = dispatch_support.find_schedule(txn, schedule_id).await?;
    let (schedule, days) = match schedule_and_days {
        Some(s) => s,
        None => {
            warn!(
                task_id,
                schedule_id, "スケジュールが見つかりません（削除済み）"
            );
            // 警告付きでタスクを完了マーク
            dispatch_support
                .mark_succeeded_with_warning(txn, task_id)
                .await?;
            return Ok(RecurringRecruitmentExecutionResult::ScheduleNotFound { schedule_id });
        }
    };

    // スケジュールが有効かチェック
    if !schedule.is_enabled {
        info!(task_id, schedule_id, "スケジュールは無効化されています");
        // 警告付きでタスクを完了マーク
        dispatch_support
            .mark_succeeded_with_warning(txn, task_id)
            .await?;
        return Ok(RecurringRecruitmentExecutionResult::ScheduleDisabled { schedule_id });
    }

    // task.schedule_datetime に対応する実行回の日時情報を復元
    let calculated_time = schedule_service
        .resolve_recruitment_time_by_recruit_start_at(&schedule, &days, task.schedule_datetime)?
        .ok_or_else(|| AppError::Business {
            message: format!(
                "task.schedule_datetime({}) に対応する定期募集時刻を解決できませんでした",
                task.schedule_datetime
            ),
        })?;

    // 出発時刻を過ぎている場合は募集作成をスキップし、次回タスクのみ登録する
    let now = Utc::now();
    if should_skip_recruitment_creation(calculated_time.quest_start_at, now) {
        return handle_past_due_recurring_schedule(
            txn,
            db_conn,
            gateway,
            dispatch_support,
            schedule_service,
            recruitment_creation_service,
            task_id,
            schedule_id,
            &schedule,
            &days,
            &calculated_time,
            now,
        )
        .await;
    }

    // マルチ募集を作成
    info!(
        task_id,
        schedule_id,
        quest_id = schedule.quest_id,
        quest_start_at = %calculated_time.quest_start_at,
        recruit_start_at = %calculated_time.recruit_start_at,
        "マルチ募集を作成します"
    );

    create_and_send_recurring_recruitment(
        txn,
        db_conn,
        gateway,
        recruitment_creation_service,
        &calculated_time,
    )
    .await?;

    info!(task_id, schedule_id, "マルチ募集を作成しました");

    // 次回実行日時を計算してscheduled_tasksに登録
    let next_task_id = dispatch_support
        .register_next_scheduled_task(txn, &schedule, &days)
        .await?;

    // 現在のタスクを正常終了にマーク
    dispatch_support.mark_succeeded(txn, task_id).await?;

    info!(task_id, schedule_id, next_task_id, "定期募集タスク実行完了");

    Ok(RecurringRecruitmentExecutionResult::Success { next_task_id })
}

/// 出発時刻を過ぎている場合の処理
///
/// 現在募集可能な回（募集開始済みかつ出発前）があれば即時実行し、無ければ
/// 募集作成をスキップして次回実行タスクのみ登録する。
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
async fn handle_past_due_recurring_schedule<
    G,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    dispatch_support: &RecurringRecruitmentDispatchSupportService<ST, RR, SR>,
    schedule_service: &RecruitmentScheduleService,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    task_id: i32,
    schedule_id: i32,
    schedule: &battle_recruitment_schedules::Model,
    days: &[battle_recruitment_schedule_days::Model],
    calculated_time: &CalculatedRecruitmentTime,
    now: chrono::DateTime<Utc>,
) -> Result<RecurringRecruitmentExecutionResult>
where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    RR: ScheduledTaskRecurringRecruitmentRepository,
    SR: BattleRecruitmentScheduleRepository,
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    // 過去タスクをスキップする前に、現在募集可能な回（募集開始済みかつ出発前）を探索
    if let Some(recoverable_time) =
        schedule_service.resolve_executable_recruitment_time_at_now(schedule, days, now)?
    {
        info!(
            task_id,
            schedule_id,
            skipped_recruit_start_at = %calculated_time.recruit_start_at,
            recover_recruit_start_at = %recoverable_time.recruit_start_at,
            recover_quest_start_at = %recoverable_time.quest_start_at,
            "過去タスクをスキップし、現在募集可能な回を即時実行します"
        );

        create_and_send_recurring_recruitment(
            txn,
            db_conn,
            gateway,
            recruitment_creation_service,
            &recoverable_time,
        )
        .await?;

        let next_task_id = dispatch_support
            .register_next_scheduled_task(txn, schedule, days)
            .await?;

        dispatch_support
            .mark_succeeded_with_warning(txn, task_id)
            .await?;

        warn!(
            task_id,
            schedule_id,
            skipped_quest_start_at = %calculated_time.quest_start_at,
            now = %now,
            next_task_id,
            "過去タスクは警告付き完了とし、現在募集可能な回を即時実行しました"
        );

        return Ok(
            RecurringRecruitmentExecutionResult::RecoveredCurrentWindow {
                schedule_id,
                next_task_id,
            },
        );
    }

    let next_task_id = dispatch_support
        .register_next_scheduled_task(txn, schedule, days)
        .await?;

    dispatch_support
        .mark_succeeded_with_warning(txn, task_id)
        .await?;

    warn!(
        task_id,
        schedule_id,
        quest_start_at = %calculated_time.quest_start_at,
        now = %now,
        next_task_id,
        "出発時刻に到達済みのため募集作成をスキップし、次回実行タスクのみ登録しました"
    );

    Ok(RecurringRecruitmentExecutionResult::SkippedPastDeparture {
        schedule_id,
        next_task_id,
    })
}

/// 定期募集の表示データを組み立て、Discordへ投稿し、DB保存する
///
/// UI組み立て（Presenter）とDiscord送信（Gateway）はFacade層の責務。
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
async fn create_and_send_recurring_recruitment<
    G,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    calculated_time: &CalculatedRecruitmentTime,
) -> Result<()>
where
    G: DiscordGateway,
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    let prepared = recruitment_creation_service
        .prepare_recruitment_from_schedule(txn, db_conn, gateway, calculated_time)
        .await?;

    let (embed_content, button_components) = build_v2_recruitment_embed_and_components(
        &prepared.battle_style_name,
        &prepared.element_emojis,
    );

    let channel_id = DiscordChannelId::new(prepared.recruitment_channel_id as u64);
    let domain_message_content = MessageContent::new()
        .with_text(&prepared.message_content)
        .with_embed(embed_content)
        .with_components(button_components);

    let sent_message_id = gateway
        .send_message(channel_id, domain_message_content)
        .await?;
    let message_id = sent_message_id.get();

    debug!(message_id = %message_id, "Discordメッセージを投稿しました");

    recruitment_creation_service
        .finalize_recruitment_from_schedule(txn, calculated_time, prepared, message_id)
        .await?;

    Ok(())
}
