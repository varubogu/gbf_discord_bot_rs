// 募集スケジュールファサード 結合テスト
//
// 対象: src/facades/recruitment/recruitment_schedule_facade.rs

use gbf_discord_bot_rs::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, TEST_USER_ID, create_test_app_state};

/// テスト用ID（スケジュールテスト専用）
const SCHED_GUILD_ID: i64 = TEST_GUILD_ID + 700;
const SCHED_USER_ID: i64 = TEST_USER_ID as i64;

// =================================================
// list_recruitment_schedules
// =================================================

/// 2-3: 正常系 - スケジュール未登録時
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_list_schedules_empty() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state);

    // 存在しないギルドIDでリスト取得
    let result = facade
        .list_recruitment_schedules(SCHED_GUILD_ID + 99, SCHED_USER_ID, true)
        .await;
    assert!(
        result.is_ok(),
        "スケジュール一覧取得に失敗: {:?}",
        result.err()
    );

    let schedules = result.unwrap();
    assert!(
        schedules.is_empty(),
        "存在しないギルドにスケジュールが返りました"
    );
}

// =================================================
// delete_recruitment_schedule
// =================================================

/// 3-4: 異常系 - 存在しないスケジュールの削除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_delete_schedule_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state);

    let result = facade
        .delete_recruitment_schedule(SCHED_GUILD_ID, 99999, SCHED_USER_ID, true)
        .await;
    assert!(
        result.is_err(),
        "存在しないスケジュールの削除でエラーが返りませんでした"
    );
}

// =================================================
// toggle_recruitment_schedule
// =================================================

/// 4-4: 異常系 - 存在しないスケジュールの切替
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_toggle_schedule_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state);

    let result = facade
        .toggle_recruitment_schedule(SCHED_GUILD_ID, 99999, SCHED_USER_ID, true)
        .await;
    assert!(
        result.is_err(),
        "存在しないスケジュールの切替でエラーが返りませんでした"
    );
}
