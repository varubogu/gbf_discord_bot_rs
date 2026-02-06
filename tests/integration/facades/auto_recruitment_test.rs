// 自動募集ファサード 結合テスト
//
// 対象:
// - src/facades/auto_recruitment/matching_check_facade.rs
// - src/facades/auto_recruitment/quest_selection_facade.rs
// - src/facades/auto_recruitment/time_selection_facade.rs
// - src/facades/auto_recruitment/status_facade.rs

use gbf_discord_bot_rs::facades::auto_recruitment::matching_check_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::quest_selection_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::status_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::time_selection_facade;
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, TEST_USER_ID, create_test_app_state};

/// テスト用ID（自動募集テスト専用）
const AUTO_GUILD_ID: u64 = (TEST_GUILD_ID + 800) as u64;
const AUTO_USER_ID: u64 = TEST_USER_ID;

// =================================================
// check_and_notify_after_quest_selection（スタブ）
// =================================================

/// 4-1: 正常系 - スタブ動作確認
#[tokio::test]
async fn test_check_and_notify_after_quest_selection_stub() {
    let result = matching_check_facade::check_and_notify_after_quest_selection(
        AUTO_GUILD_ID,
        AUTO_USER_ID,
        vec![1, 2, 3],
    )
    .await;

    assert!(
        result.is_ok(),
        "スタブ関数がエラーを返しました: {:?}",
        result.err()
    );
}

// =================================================
// check_and_notify_after_time_selection（スタブ）
// =================================================

/// 5-1: 正常系 - スタブ動作確認
#[tokio::test]
async fn test_check_and_notify_after_time_selection_stub() {
    let result = matching_check_facade::check_and_notify_after_time_selection(
        AUTO_GUILD_ID,
        AUTO_USER_ID,
        1,
        15,
        vec![21, 22, 23],
    )
    .await;

    assert!(
        result.is_ok(),
        "スタブ関数がエラーを返しました: {:?}",
        result.err()
    );
}

// =================================================
// handle_quest_selection
// =================================================

/// 1-3: 異常系 - 自動募集未登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_quest_selection_not_registered() {
    let app_state = Arc::new(create_test_app_state().await);

    // 自動募集が登録されていないギルドでクエスト選択
    let result = quest_selection_facade::handle_quest_selection(
        &app_state,
        AUTO_GUILD_ID + 99,
        AUTO_USER_ID,
        vec![1, 2],
    )
    .await;

    assert!(result.is_err(), "自動募集未登録でエラーが返りませんでした");
}

// =================================================
// handle_time_selection
// =================================================

/// 2-4: 異常系 - 自動募集未登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_time_selection_not_registered() {
    let app_state = Arc::new(create_test_app_state().await);

    // 自動募集が登録されていないギルドで時間帯選択
    let result = time_selection_facade::handle_time_selection(
        &app_state,
        AUTO_GUILD_ID + 99,
        AUTO_USER_ID,
        1,
        15,
        vec![21, 22],
    )
    .await;

    assert!(result.is_err(), "自動募集未登録でエラーが返りませんでした");
}

// =================================================
// get_participation_status
// =================================================

/// 3-3: 異常系 - 自動募集未登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_participation_status_not_registered() {
    let app_state = Arc::new(create_test_app_state().await);

    // 自動募集が登録されていないギルドで参加状況取得
    let result =
        status_facade::get_participation_status(&app_state, AUTO_GUILD_ID + 99, AUTO_USER_ID).await;

    assert!(result.is_err(), "自動募集未登録でエラーが返りませんでした");
}
