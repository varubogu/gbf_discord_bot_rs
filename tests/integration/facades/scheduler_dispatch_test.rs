// スケジューラータスクディスパッチファサード 結合テスト
//
// 対象: src/facades/schedule/scheduler_task_dispatch_facade/
//
// ステップ5・8でservice層（TaskDispatchService／RecurringRecruitmentTaskExecutor／
// AutoMatchingTaskExecutor）からfacade層へ移設したディスパッチロジックを、
// 公開API `SchedulerDispatchUseCase::dispatch_due_tasks` 経由で検証する。
// 移設時点ではこの経路を直接検証するテストが存在しなかったため、
// task_typeごとの振り分けと、定期募集／自動マッチングの入口の分岐をここで担保する。
//
// 注意: `dispatch_due_tasks` は worker.scheduled_tasks 全体を対象に動作するため、
// 各テストは自身が挿入したタスクのみを対象に検証し、前後で後片付けを行う。
// CIでは `RUST_TEST_THREADS=1` で直列実行される。

use chrono::{Duration, Utc};
use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::facades::schedule::SchedulerTaskDispatchFacade;
use gbf_discord_bot_rs::infrastructure::database::repositories::{
    SeaOrmBattleRecruitmentsRepository, SeaOrmGuildMessageTextRepository,
    SeaOrmMessageTextRepository, SeaOrmRecruitmentParticipantsRepository,
};
use gbf_discord_bot_rs::models::entities::guild_master::battle_recruitment_schedules;
use gbf_discord_bot_rs::models::entities::worker::scheduled_tasks::{
    ScheduledTaskType, TaskExecutionStatus,
};
use gbf_discord_bot_rs::models::entities::worker::{
    scheduled_task_recurring_recruitments, scheduled_tasks,
};
use gbf_discord_bot_rs::services::schedule::SchedulerDispatchUseCase;
use gbf_discord_bot_rs::types::AppState;
use sea_orm::prelude::TimeTime;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use super::test_helper::{MockTestGateway, TEST_GUILD_ID, create_test_app_state};

/// テスト用ギルドID（ディスパッチテスト専用）
const DISPATCH_GUILD_ID: i64 = TEST_GUILD_ID + 900;

/// ディスパッチテストで利用するファサードの具象型
type TestDispatchFacade = SchedulerTaskDispatchFacade<
    SeaOrmBattleRecruitmentsRepository,
    SeaOrmRecruitmentParticipantsRepository,
    SeaOrmGuildMessageTextRepository,
    SeaOrmMessageTextRepository,
>;

/// `main.rs` の `start_scheduler` と同じ組み立てでファサードを構築する
fn build_dispatch_facade(app_state: &AppState) -> TestDispatchFacade {
    let repos = app_state.repositories;
    SchedulerTaskDispatchFacade::new(
        app_state.system_db.clone(),
        Arc::new(repos.battle_recruitments),
        Arc::new(repos.recruitment_participants),
        app_state.message_service.clone(),
        repos,
    )
}

/// Discordへ一切送信しないことを期待するゲートウェイモック
fn gateway_expecting_no_send() -> Arc<MockTestGateway> {
    let mut gateway = MockTestGateway::new();
    gateway.expect_send_message().times(0);
    Arc::new(gateway)
}

/// 指定ギルドのタスクと、それに紐づく定期募集情報を削除する
async fn cleanup_tasks_for_guild(db: &DatabaseConnection, guild_id: i64) {
    let task_ids: Vec<i32> = scheduled_tasks::Entity::find()
        .filter(scheduled_tasks::Column::GuildId.eq(guild_id))
        .all(db)
        .await
        .map(|tasks| tasks.into_iter().map(|task| task.id).collect())
        .unwrap_or_default();

    if !task_ids.is_empty() {
        let _ = scheduled_task_recurring_recruitments::Entity::delete_many()
            .filter(
                scheduled_task_recurring_recruitments::Column::ScheduledTaskId
                    .is_in(task_ids.clone()),
            )
            .exec(db)
            .await;
    }

    let _ = scheduled_tasks::Entity::delete_many()
        .filter(scheduled_tasks::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// 自動マッチングタスク（guild_id=None）を全て削除する
///
/// 自動マッチングは実行のたびに次回タスクを登録して連鎖するため、
/// テストの前後で必ず片付ける。
async fn cleanup_auto_matching_tasks(db: &DatabaseConnection) {
    let _ = scheduled_tasks::Entity::delete_many()
        .filter(scheduled_tasks::Column::TaskType.eq(ScheduledTaskType::AutoMatching.as_i32()))
        .exec(db)
        .await;
}

/// 指定ギルドの定期募集スケジュールを削除する
async fn cleanup_schedules(db: &DatabaseConnection, guild_id: i64) {
    let _ = battle_recruitment_schedules::Entity::delete_many()
        .filter(battle_recruitment_schedules::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// 実行待ちタスクを1件挿入する
async fn insert_pending_task(
    db: &DatabaseConnection,
    task_type: i32,
    guild_id: Option<i64>,
    schedule_datetime: chrono::DateTime<Utc>,
) -> scheduled_tasks::Model {
    scheduled_tasks::ActiveModel {
        schedule_datetime: Set(schedule_datetime),
        task_type: Set(task_type),
        guild_id: Set(guild_id),
        channel_id: Set(None),
        execution_status: Set(TaskExecutionStatus::Pending),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("タスクの挿入に失敗")
}

/// タスクの実行状態を取得する
async fn task_status(db: &DatabaseConnection, task_id: i32) -> TaskExecutionStatus {
    scheduled_tasks::Entity::find_by_id(task_id)
        .one(db)
        .await
        .expect("タスク取得に失敗")
        .expect("タスクが存在しない")
        .execution_status
}

// =================================================
// task_type=4: 定期募集タスク（run_recurring_recruitment_dispatch）
// =================================================

// NOTE: `run_recurring_recruitment_dispatch` の `ScheduleNotFound` 分岐
// （スケジュール削除済み）は、`worker.scheduled_task_recurring_recruitments.schedule_id`
// が `ON DELETE CASCADE` でスケジュール本体を参照しているため、
// スケジュールを削除すると紐づけ行も同時に消える。
// 「紐づけ行だけが残る」状態はスキーマ上作れず到達不能なので、テストは用意しない。

/// 1-1: スケジュールが無効化されている場合、警告付き完了としてマークされる
#[tokio::test]
async fn test_dispatch_recurring_recruitment_warns_when_schedule_disabled() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();
    let guild_id = DISPATCH_GUILD_ID + 2;

    cleanup_tasks_for_guild(db, guild_id).await;
    cleanup_schedules(db, guild_id).await;

    GuildManagementFacade::new(Arc::new(app_state.clone()))
        .register_new_guild(guild_id, "ディスパッチテストギルド")
        .await
        .expect("ギルド登録に失敗");

    let schedule = battle_recruitment_schedules::ActiveModel {
        name: Set("無効化された定期募集".to_string()),
        guild_id: Set(guild_id),
        channel_id: Set(guild_id + 10_000),
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_time: Set(TimeTime::from_hms(20, 0, 0).expect("時刻生成に失敗")),
        recruit_start_day_offset: Set(0),
        recruit_start_time: Set(Some(TimeTime::from_hms(19, 0, 0).expect("時刻生成に失敗"))),
        max_participants: Set(None),
        note: Set(None),
        is_enabled: Set(false),
        created_by: Set(guild_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("スケジュールの挿入に失敗");

    let task = insert_pending_task(
        db,
        ScheduledTaskType::RecurringRecruitment.as_i32(),
        Some(guild_id),
        Utc::now() - Duration::minutes(1),
    )
    .await;

    scheduled_task_recurring_recruitments::ActiveModel {
        scheduled_task_id: Set(task.id),
        recruitment_schedule_id: Set(schedule.id),
    }
    .insert(db)
    .await
    .expect("定期募集情報の挿入に失敗");

    build_dispatch_facade(&app_state)
        .dispatch_due_tasks(&gateway_expecting_no_send())
        .await
        .expect("ディスパッチに失敗");

    assert_eq!(
        task_status(db, task.id).await,
        TaskExecutionStatus::SucceededWithWarning,
        "無効化されたスケジュールのタスクは警告付き完了になること"
    );

    cleanup_tasks_for_guild(db, guild_id).await;
    cleanup_schedules(db, guild_id).await;
}

/// 1-2: 定期募集情報が紐づいていない場合、失敗としてマークされる
#[tokio::test]
async fn test_dispatch_recurring_recruitment_fails_without_recurring_record() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();
    let guild_id = DISPATCH_GUILD_ID + 3;

    cleanup_tasks_for_guild(db, guild_id).await;

    // scheduled_task_recurring_recruitments を意図的に作らない（データ不整合）
    let task = insert_pending_task(
        db,
        ScheduledTaskType::RecurringRecruitment.as_i32(),
        Some(guild_id),
        Utc::now() - Duration::minutes(1),
    )
    .await;

    build_dispatch_facade(&app_state)
        .dispatch_due_tasks(&gateway_expecting_no_send())
        .await
        .expect("ディスパッチに失敗");

    assert_eq!(
        task_status(db, task.id).await,
        TaskExecutionStatus::Failed,
        "定期募集情報が欠落したタスクは失敗になること"
    );

    cleanup_tasks_for_guild(db, guild_id).await;
}

// =================================================
// task_type=7: 自動マッチングタスク（run_auto_matching_dispatch）
// =================================================

/// 2-1: マッチング対象が無い場合でも正常終了し、次回タスクが登録される
#[tokio::test]
async fn test_dispatch_auto_matching_succeeds_and_registers_next_task() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    cleanup_auto_matching_tasks(db).await;

    let task = insert_pending_task(
        db,
        ScheduledTaskType::AutoMatching.as_i32(),
        None,
        Utc::now() - Duration::minutes(1),
    )
    .await;

    build_dispatch_facade(&app_state)
        .dispatch_due_tasks(&gateway_expecting_no_send())
        .await
        .expect("ディスパッチに失敗");

    assert_eq!(
        task_status(db, task.id).await,
        TaskExecutionStatus::Succeeded,
        "マッチング対象が無くてもタスクは正常終了すること"
    );

    // 次回の自動マッチングタスクが新規登録されていること
    let next_tasks = scheduled_tasks::Entity::find()
        .filter(scheduled_tasks::Column::TaskType.eq(ScheduledTaskType::AutoMatching.as_i32()))
        .filter(scheduled_tasks::Column::Id.ne(task.id))
        .all(db)
        .await
        .expect("次回タスクの取得に失敗");

    assert_eq!(
        next_tasks.len(),
        1,
        "次回の自動マッチングタスクが1件登録されること"
    );
    assert_eq!(
        next_tasks[0].execution_status,
        TaskExecutionStatus::Pending,
        "次回タスクは実行待ちであること"
    );
    assert!(
        next_tasks[0].schedule_datetime > task.schedule_datetime,
        "次回タスクは元タスクより後に予定されること"
    );

    cleanup_auto_matching_tasks(db).await;
}

// =================================================
// ディスパッチ全体の振り分け（run_dispatch_cycle）
// =================================================

/// 3-1: 実行時刻に達していないタスクは実行されない
///
/// プリロード範囲（現在時刻+20秒）には入るが実行時刻には達していないタスクが、
/// `is_task_due` の判定で見送られることを確認する。
#[tokio::test]
async fn test_dispatch_skips_task_not_yet_due() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();
    let guild_id = DISPATCH_GUILD_ID + 4;

    cleanup_tasks_for_guild(db, guild_id).await;

    let task = insert_pending_task(
        db,
        ScheduledTaskType::RecurringRecruitment.as_i32(),
        Some(guild_id),
        Utc::now() + Duration::seconds(15),
    )
    .await;

    build_dispatch_facade(&app_state)
        .dispatch_due_tasks(&gateway_expecting_no_send())
        .await
        .expect("ディスパッチに失敗");

    assert_eq!(
        task_status(db, task.id).await,
        TaskExecutionStatus::Pending,
        "実行時刻前のタスクは実行待ちのままであること"
    );

    cleanup_tasks_for_guild(db, guild_id).await;
}

/// 3-2: 未知のtask_typeは失敗としてマークされる
#[tokio::test]
async fn test_dispatch_marks_unknown_task_type_as_failed() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();
    let guild_id = DISPATCH_GUILD_ID + 5;

    cleanup_tasks_for_guild(db, guild_id).await;

    let task = insert_pending_task(db, 99, Some(guild_id), Utc::now() - Duration::minutes(1)).await;

    build_dispatch_facade(&app_state)
        .dispatch_due_tasks(&gateway_expecting_no_send())
        .await
        .expect("ディスパッチに失敗");

    assert_eq!(
        task_status(db, task.id).await,
        TaskExecutionStatus::Failed,
        "未知のtask_typeは失敗になること"
    );

    cleanup_tasks_for_guild(db, guild_id).await;
}
