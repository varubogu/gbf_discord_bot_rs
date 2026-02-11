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
use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::models::entities::guild_master::{
    auto_recruitment_participants, auto_recruitments, guilds, user_desired_quests,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, TEST_USER_ID, create_test_app_state};

/// テスト用ID（自動募集テスト専用）
const AUTO_GUILD_ID: u64 = (TEST_GUILD_ID + 800) as u64;
const AUTO_USER_ID: u64 = TEST_USER_ID;

/// 自動募集関連のテストデータを削除
async fn cleanup_auto_recruitment_data(
    db: &sea_orm::DatabaseConnection,
    guild_id: i64,
    user_id: i64,
) {
    let _ = user_desired_quests::Entity::delete_many()
        .filter(user_desired_quests::Column::GuildId.eq(guild_id))
        .filter(user_desired_quests::Column::UserId.eq(user_id))
        .exec(db)
        .await;

    let _ = auto_recruitment_participants::Entity::delete_many()
        .filter(auto_recruitment_participants::Column::GuildId.eq(guild_id))
        .filter(auto_recruitment_participants::Column::UserId.eq(user_id))
        .exec(db)
        .await;

    let _ = auto_recruitments::Entity::delete_many()
        .filter(auto_recruitments::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;

    let _ = guilds::Entity::delete_many()
        .filter(guilds::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// 自動募集登録済みの前提データを作成
async fn setup_auto_recruitment_registered(
    app_state: &Arc<gbf_discord_bot_rs::types::AppState>,
    guild_id: i64,
) {
    let guild_facade = GuildManagementFacade::new(app_state.clone());
    guild_facade
        .register_new_guild(guild_id, "自動募集テストギルド")
        .await
        .unwrap();

    let model = auto_recruitments::ActiveModel {
        guild_id: Set(guild_id),
        category_id: Set(guild_id + 10),
        matching_channel_id: Set(Some(guild_id + 11)),
        quest_channel_id: Set(Some(guild_id + 12)),
        matching_channel_is_bot_created: Set(true),
        quest_channel_is_bot_created: Set(true),
        matching_message_id: Set(Some(guild_id + 13)),
        days_range: Set(3),
        ..Default::default()
    };

    model.insert(app_state.guild_db()).await.unwrap();
}

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

/// 1-1: 正常系 - クエスト選択登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_quest_selection_register_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 1) as i64;
    let user_id = (AUTO_USER_ID + 1) as i64;
    let quest_ids = vec![1, 2];

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    let result = quest_selection_facade::handle_quest_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        quest_ids.clone(),
    )
    .await;

    assert!(result.is_ok(), "クエスト選択登録に失敗: {:?}", result.err());
    match result.unwrap() {
        quest_selection_facade::QuestSelectionResult::Registered {
            quest_ids: returned_quest_ids,
        } => {
            assert_eq!(returned_quest_ids, quest_ids);
        }
    }

    let mut saved_quests = user_desired_quests::Entity::find()
        .filter(user_desired_quests::Column::GuildId.eq(guild_id))
        .filter(user_desired_quests::Column::UserId.eq(user_id))
        .all(app_state.guild_db())
        .await
        .unwrap();

    assert_eq!(saved_quests.len(), quest_ids.len());
    assert!(
        saved_quests.iter().all(|q| q.battle_style_id == 0),
        "battle_style_id=0で保存されていません"
    );

    saved_quests.sort_by_key(|q| q.quest_id);
    let mut expected_quests = quest_ids.clone();
    expected_quests.sort_unstable();
    let actual_quest_ids: Vec<i32> = saved_quests.iter().map(|q| q.quest_id).collect();
    assert_eq!(actual_quest_ids, expected_quests);

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

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
