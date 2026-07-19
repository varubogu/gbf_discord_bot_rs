// 募集スケジュールファサード 結合テスト
//
// 対象: src/facades/recruitment/recruitment_schedule_facade.rs

use gbf_discord_bot_rs::facades::channel::channel_management_facade::ChannelManagementFacade;
use gbf_discord_bot_rs::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use gbf_discord_bot_rs::facades::recruitment::recruitment_schedule_list::get_schedules_for_autocomplete;
use gbf_discord_bot_rs::models::entities::guild_master::battle_recruitment_schedules;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, TEST_USER_ID, create_test_app_state};

/// テスト用ID（スケジュールテスト専用）
const SCHED_GUILD_ID: i64 = TEST_GUILD_ID + 700;
const SCHED_USER_ID: i64 = TEST_USER_ID as i64;

/// 定期募集の作成に必須なマルチ募集チャンネルを登録する。
async fn setup_schedule_prerequisites(
    app_state: &Arc<gbf_discord_bot_rs::types::AppState>,
    guild_id: i64,
) {
    ChannelManagementFacade::new(app_state.clone())
        .register_channel(
            guild_id,
            "スケジュールテストギルド".to_string(),
            2,
            guild_id + 10_000,
        )
        .await
        .unwrap();
}

/// テスト用スケジュールデータを削除
async fn cleanup_schedules(db: &sea_orm::DatabaseConnection, guild_id: i64, user_id: i64) {
    let _ = battle_recruitment_schedules::Entity::delete_many()
        .filter(battle_recruitment_schedules::Column::GuildId.eq(guild_id))
        .filter(battle_recruitment_schedules::Column::CreatedBy.eq(user_id))
        .exec(db)
        .await;
}

// =================================================
// create_recruitment_schedule
// =================================================

/// 1-1: 正常系 - 基本的なスケジュール作成
#[tokio::test]
async fn test_create_schedule_basic() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 1) as u64;
    let user_id = (SCHED_USER_ID + 1) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    let result = facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "基本スケジュール".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await;

    assert!(result.is_ok(), "スケジュール作成に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 1-2: 正常系 - battle_style_id指定あり
#[tokio::test]
async fn test_create_schedule_with_battle_style() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 2) as u64;
    let user_id = (SCHED_USER_ID + 2) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    let result = facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "攻略方法指定".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            Some(1), // battle_style_id指定
            Some(0),
            None,
            None,
        )
        .await;

    assert!(
        result.is_ok(),
        "battle_style_id指定でスケジュール作成に失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 1-3: 正常系 - note指定あり
#[tokio::test]
async fn test_create_schedule_with_note() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 3) as u64;
    let user_id = (SCHED_USER_ID + 3) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    let result = facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "備考付き".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            Some("初心者歓迎".to_string()),
            None,
        )
        .await;

    assert!(
        result.is_ok(),
        "note指定でスケジュール作成に失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 1-4: 正常系 - dismissal_times指定あり
#[tokio::test]
async fn test_create_schedule_with_dismissal_times() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 4) as u64;
    let user_id = (SCHED_USER_ID + 4) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    let result = facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "解散時刻付き".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            Some("21:00".to_string()), // dismissal_times指定
        )
        .await;

    assert!(
        result.is_ok(),
        "dismissal_times指定でスケジュール作成に失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 1-5: 異常系 - 存在しないquest_alias
#[tokio::test]
async fn test_create_schedule_quest_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    setup_schedule_prerequisites(&app_state, SCHED_GUILD_ID).await;
    let facade = RecruitmentScheduleFacade::new(app_state);

    let result = facade
        .create_recruitment_schedule(
            SCHED_GUILD_ID as u64,
            SCHED_USER_ID as u64,
            "テストスケジュール".to_string(),
            "存在しないクエスト名",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "存在しないクエストでエラーが返りませんでした"
    );
}

/// 1-6: 異常系 - 無効な時刻フォーマット
#[tokio::test]
async fn test_create_schedule_invalid_time_format() {
    let app_state = Arc::new(create_test_app_state().await);
    setup_schedule_prerequisites(&app_state, SCHED_GUILD_ID).await;
    let facade = RecruitmentScheduleFacade::new(app_state);

    let result = facade
        .create_recruitment_schedule(
            SCHED_GUILD_ID as u64,
            SCHED_USER_ID as u64,
            "テストスケジュール".to_string(),
            "ルシファーHL",
            "invalid_time",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "無効な時刻フォーマットでエラーが返りませんでした"
    );
}

// =================================================
// list_recruitment_schedules
// =================================================

/// 2-1: 正常系 - 全スケジュール表示
#[tokio::test]
async fn test_list_all_schedules() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 10) as u64;
    let user_id = (SCHED_USER_ID + 10) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    // スケジュールを作成
    facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "テストスケジュール".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 全スケジュール取得（own_only=false）
    let result = facade
        .list_recruitment_schedules(guild_id as i64, user_id as i64, false)
        .await;
    assert!(
        result.is_ok(),
        "全スケジュール取得に失敗: {:?}",
        result.err()
    );

    let schedules = result.unwrap();
    assert!(!schedules.is_empty(), "スケジュールが取得できませんでした");

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 2-2: 正常系 - 自分のスケジュールのみ表示
#[tokio::test]
async fn test_list_own_schedules() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 11) as u64;
    let user_id = (SCHED_USER_ID + 11) as u64;

    setup_schedule_prerequisites(&app_state, guild_id as i64).await;
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;

    // スケジュールを作成
    facade
        .create_recruitment_schedule(
            guild_id,
            user_id,
            "マイスケジュール".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 自分のスケジュールのみ取得（own_only=true）
    let result = facade
        .list_recruitment_schedules(guild_id as i64, user_id as i64, true)
        .await;
    assert!(
        result.is_ok(),
        "自分のスケジュール取得に失敗: {:?}",
        result.err()
    );

    let schedules = result.unwrap();
    assert!(!schedules.is_empty(), "スケジュールが取得できませんでした");
    // 全て自分が作成したスケジュールであることを確認
    assert!(schedules.iter().all(|s| s.created_by == user_id as i64));

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id as i64, user_id as i64).await;
}

/// 2-3: 正常系 - スケジュール未登録時
#[tokio::test]
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

/// 3-1: 正常系 - 作成者による削除
#[tokio::test]
async fn test_delete_schedule_by_creator() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 20) as i64;
    let user_id = (SCHED_USER_ID + 20) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;

    // スケジュールを作成
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            user_id as u64,
            "削除テスト".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 作成者が削除（is_admin=false）
    let result = facade
        .delete_recruitment_schedule(guild_id, created.schedule_id as i32, user_id, false)
        .await;
    assert!(result.is_ok(), "作成者による削除に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;
}

/// 3-2: 正常系 - 管理者による削除
#[tokio::test]
async fn test_delete_schedule_by_admin() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 21) as i64;
    let creator_id = (SCHED_USER_ID + 21) as i64;
    let admin_id = (SCHED_USER_ID + 22) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;

    // スケジュールを作成
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            creator_id as u64,
            "管理者削除テスト".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 管理者が削除（is_admin=true）
    let result = facade
        .delete_recruitment_schedule(guild_id, created.schedule_id as i32, admin_id, true)
        .await;
    assert!(result.is_ok(), "管理者による削除に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;
}

/// 3-3: 異常系 - 権限のない削除
#[tokio::test]
async fn test_delete_schedule_without_permission() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 23) as i64;
    let creator_id = (SCHED_USER_ID + 23) as i64;
    let other_user_id = (SCHED_USER_ID + 24) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;

    // スケジュールを作成
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            creator_id as u64,
            "権限テスト".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 他のユーザーが削除を試みる（is_admin=false）
    let result = facade
        .delete_recruitment_schedule(guild_id, created.schedule_id as i32, other_user_id, false)
        .await;
    assert!(
        result.is_err(),
        "権限のないユーザーが削除できてしまいました"
    );

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;
}

/// 3-4: 異常系 - 存在しないスケジュールの削除
#[tokio::test]
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

/// 4-1: 正常系 - 有効→無効への切替
#[tokio::test]
async fn test_toggle_schedule_enable_to_disable() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 30) as i64;
    let user_id = (SCHED_USER_ID + 30) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;

    // スケジュールを作成（デフォルトで有効）
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            user_id as u64,
            "切替テスト1".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 無効に切替
    let result = facade
        .toggle_recruitment_schedule(guild_id, created.schedule_id as i32, user_id, false)
        .await;
    assert!(result.is_ok(), "有効→無効の切替に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;
}

/// 4-2: 正常系 - 無効→有効への切替
#[tokio::test]
async fn test_toggle_schedule_disable_to_enable() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 31) as i64;
    let user_id = (SCHED_USER_ID + 31) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;

    // スケジュールを作成
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            user_id as u64,
            "切替テスト2".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 無効に切替
    facade
        .toggle_recruitment_schedule(guild_id, created.schedule_id as i32, user_id, false)
        .await
        .unwrap();

    // 有効に切替
    let result = facade
        .toggle_recruitment_schedule(guild_id, created.schedule_id as i32, user_id, false)
        .await;
    assert!(result.is_ok(), "無効→有効の切替に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;
}

/// 4-3: 正常系 - 管理者による切替
#[tokio::test]
async fn test_toggle_schedule_by_admin() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 32) as i64;
    let creator_id = (SCHED_USER_ID + 32) as i64;
    let admin_id = (SCHED_USER_ID + 33) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;

    // スケジュールを作成
    let created = facade
        .create_recruitment_schedule(
            guild_id as u64,
            creator_id as u64,
            "管理者切替テスト".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // 管理者が切替（is_admin=true）
    let result = facade
        .toggle_recruitment_schedule(guild_id, created.schedule_id as i32, admin_id, true)
        .await;
    assert!(result.is_ok(), "管理者による切替に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, creator_id).await;
}

/// 4-4: 異常系 - 存在しないスケジュールの切替
#[tokio::test]
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

// =================================================
// get_schedules_for_autocomplete
// =================================================

/// 5-1: 正常系 - 自分のスケジュール候補取得
#[tokio::test]
async fn test_get_schedules_for_autocomplete_with_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let guild_id = (SCHED_GUILD_ID + 40) as i64;
    let user_id = (SCHED_USER_ID + 40) as i64;

    setup_schedule_prerequisites(&app_state, guild_id).await;
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;

    // スケジュールを作成
    facade
        .create_recruitment_schedule(
            guild_id as u64,
            user_id as u64,
            "オートコンプリートテスト".to_string(),
            "アルバハHL",
            "20:00",
            "月火水",
            "19:00",
            None,
            Some(0),
            None,
            None,
        )
        .await
        .unwrap();

    // オートコンプリート候補取得
    let options = get_schedules_for_autocomplete(&app_state, guild_id, user_id).await;

    assert!(!options.is_empty(), "オートコンプリート候補が空です");

    // クリーンアップ
    cleanup_schedules(app_state.guild_db(), guild_id, user_id).await;
}

/// 5-2: 正常系 - スケジュール未登録時
#[tokio::test]
async fn test_get_schedules_for_autocomplete_empty() {
    let app_state = Arc::new(create_test_app_state().await);

    // 存在しないギルド・ユーザーでオートコンプリート候補取得
    let options =
        get_schedules_for_autocomplete(&app_state, SCHED_GUILD_ID + 99, SCHED_USER_ID).await;

    assert!(options.is_empty(), "スケジュールがないのに候補が返りました");
}
