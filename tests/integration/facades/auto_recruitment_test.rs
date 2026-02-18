// 自動募集ファサード 結合テスト
//
// 対象:
// - src/facades/auto_recruitment/matching_check_facade.rs
// - src/facades/auto_recruitment/quest_selection_facade.rs
// - src/facades/auto_recruitment/time_selection_facade.rs
// - src/facades/auto_recruitment/status_facade.rs

use gbf_discord_bot_rs::facades::auto_recruitment::category_setup_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::matching_check_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::quest_selection_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::status_facade;
use gbf_discord_bot_rs::facades::auto_recruitment::time_selection_facade;
use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::models::entities::guild_master::{
    auto_recruitment_channels, auto_recruitment_participants, auto_recruitment_quest_messages,
    auto_recruitments, guilds, user_desired_quests,
};
use gbf_discord_bot_rs::types::discord::{
    ChannelData, ChannelKind, DiscordChannelId, DiscordGuildId,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use super::test_helper::{MockTestGateway, TEST_GUILD_ID, TEST_USER_ID, create_test_app_state};

/// テスト用ID（自動募集テスト専用）
const AUTO_GUILD_ID: u64 = (TEST_GUILD_ID + 800) as u64;
const AUTO_USER_ID: u64 = TEST_USER_ID;

/// DB環境変数が不足している場合はテストをスキップする
fn should_skip_for_missing_db_env() -> bool {
    let (available, missing) = gbf_discord_bot_rs::test_utils::check_database_availability();
    if !available {
        println!("テストをスキップします（DB接続情報不足）: {:?}", missing);
        return true;
    }
    false
}

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

/// 1-2: 正常系 - クエスト選択の上書き
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_quest_selection_overwrite() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 50) as i64;
    let user_id = (AUTO_USER_ID + 50) as i64;
    let quest_ids_1 = vec![1, 2];
    let quest_ids_2 = vec![3, 4];

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 1回目のクエスト選択
    quest_selection_facade::handle_quest_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        quest_ids_1,
    )
    .await
    .unwrap();

    // 2回目のクエスト選択（上書き）
    let result = quest_selection_facade::handle_quest_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        quest_ids_2,
    )
    .await;

    assert!(
        result.is_ok(),
        "クエスト選択の上書きに失敗: {:?}",
        result.err()
    );

    // クリーンアップ
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

/// 2-1: 正常系 - 時間帯選択登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_time_selection_register_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 10) as i64;
    let user_id = (AUTO_USER_ID + 10) as i64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 時間帯選択を登録
    let result = time_selection_facade::handle_time_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        1,                // 月
        15,               // 日
        vec![21, 22, 23], // 21:00, 22:00, 23:00
    )
    .await;

    assert!(result.is_ok(), "時間帯選択登録に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

/// 2-2: 正常系 - 時間帯選択の上書き
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_time_selection_overwrite() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 11) as i64;
    let user_id = (AUTO_USER_ID + 11) as i64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 1回目の時間帯選択
    time_selection_facade::handle_time_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        1,
        15,
        vec![21, 22],
    )
    .await
    .unwrap();

    // 2回目の時間帯選択（上書き）
    let result = time_selection_facade::handle_time_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        1,
        15,
        vec![23, 24], // 異なる時間帯
    )
    .await;

    assert!(
        result.is_ok(),
        "時間帯選択の上書きに失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

/// 2-3: 正常系 - 複数時間帯の選択
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_time_selection_multiple_times() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 12) as i64;
    let user_id = (AUTO_USER_ID + 12) as i64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 複数の時間帯を選択（最大24個まで選択可能）
    let time_slots: Vec<i32> = (5..=23).collect(); // 5:00〜23:00
    let result = time_selection_facade::handle_time_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        1,
        15,
        time_slots,
    )
    .await;

    assert!(result.is_ok(), "複数時間帯選択に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

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

/// 3-1: 正常系 - クエスト・時間帯あり
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_participation_status_with_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 20) as i64;
    let user_id = (AUTO_USER_ID + 20) as i64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // クエスト選択を登録
    quest_selection_facade::handle_quest_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        vec![1, 2],
    )
    .await
    .unwrap();

    // 時間帯選択を登録
    time_selection_facade::handle_time_selection(
        &app_state,
        guild_id as u64,
        user_id as u64,
        1,
        15,
        vec![21, 22],
    )
    .await
    .unwrap();

    // 参加状況を取得
    let result =
        status_facade::get_participation_status(&app_state, guild_id as u64, user_id as u64).await;

    assert!(result.is_ok(), "参加状況取得に失敗: {:?}", result.err());

    let status = result.unwrap();
    assert!(!status.quest_ids.is_empty(), "選択されたクエストが空です");
    assert!(!status.time_slots.is_empty(), "選択された時間帯が空です");

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

/// 3-2: 正常系 - 選択なし
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_participation_status_without_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (AUTO_GUILD_ID + 21) as i64;
    let user_id = (AUTO_USER_ID + 21) as i64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 何も選択していない状態で参加状況を取得
    let result =
        status_facade::get_participation_status(&app_state, guild_id as u64, user_id as u64).await;

    assert!(result.is_ok(), "参加状況取得に失敗: {:?}", result.err());

    let status = result.unwrap();
    assert!(
        status.quest_ids.is_empty(),
        "選択されたクエストが空ではありません"
    );
    assert!(
        status.time_slots.is_empty(),
        "選択された時間帯が空ではありません"
    );

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, user_id).await;
}

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

// =================================================
// register_category
// =================================================

/// 6-2: 異常系 - days範囲外（1以下）
#[tokio::test]
async fn test_register_category_days_too_small() {
    if should_skip_for_missing_db_env() {
        return;
    }

    let app_state = Arc::new(create_test_app_state().await);
    let mock_gateway = MockTestGateway::new();

    let guild_id = (AUTO_GUILD_ID + 200) as u64;
    let category_id = guild_id + 10;

    // days=1で登録を試みる（範囲外）
    let result = category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id,
        category_id,
        1, // 範囲外（2〜7のみ有効）
        None,
        None,
    )
    .await;

    assert!(result.is_err(), "days=1でエラーが返りませんでした");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("2〜7日の範囲"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}

/// 6-3: 異常系 - days範囲外（8以上）
#[tokio::test]
async fn test_register_category_days_too_large() {
    if should_skip_for_missing_db_env() {
        return;
    }

    let app_state = Arc::new(create_test_app_state().await);
    let mock_gateway = MockTestGateway::new();

    let guild_id = (AUTO_GUILD_ID + 201) as u64;
    let category_id = guild_id + 10;

    // days=8で登録を試みる（範囲外）
    let result = category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id,
        category_id,
        8, // 範囲外（2〜7のみ有効）
        None,
        None,
    )
    .await;

    assert!(result.is_err(), "days=8でエラーが返りませんでした");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("2〜7日の範囲"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}

/// 6-4: 異常系 - 既に登録済み
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_category_already_registered() {
    let app_state = Arc::new(create_test_app_state().await);
    let mock_gateway = MockTestGateway::new();

    let guild_id = (AUTO_GUILD_ID + 202) as i64;

    // 既存の自動募集データを削除
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // 自動募集を事前登録
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    let category_id = (guild_id + 20) as u64;

    // 既に登録されている状態で再度登録を試みる
    let result = category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await;

    assert!(result.is_err(), "既に登録済みでエラーが返りませんでした");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("既に自動募集が登録されています"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

/// 6-1: 正常系 - カテゴリ登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_category_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();

    // チャンネル作成のモック（マッチング、日付×3、クエスト選択）
    use gbf_discord_bot_rs::types::discord::DiscordChannelId;
    let mut call_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        call_count += 1;
        Ok(DiscordChannelId::new(10000 + call_count as u64))
    });

    // チャンネル編集のモック
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));

    // メッセージ送信のモック
    use gbf_discord_bot_rs::types::discord::DiscordMessageId;
    mock_gateway
        .expect_send_message()
        .returning(|_, _| Ok(DiscordMessageId::new(20000)));

    let guild_id = (AUTO_GUILD_ID + 400) as i64;
    let category_id = (guild_id + 10) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // カテゴリ登録
    let result = category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3, // 3日分
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "カテゴリ登録に失敗: {:?}", result.err());

    let reg_result = result.unwrap();
    assert_eq!(
        reg_result.category_id, category_id,
        "カテゴリIDが期待と異なります"
    );
    assert_eq!(
        reg_result.channel_count, 3,
        "作成されたチャンネル数が期待と異なります"
    );

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

// =================================================
// unregister_category
// =================================================

/// 7-1: 正常系 - カテゴリ登録解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_category_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();

    // 登録時のモック設定
    use gbf_discord_bot_rs::types::discord::{DiscordChannelId, DiscordMessageId};
    let mut call_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        call_count += 1;
        Ok(DiscordChannelId::new(10000 + call_count as u64))
    });
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));
    mock_gateway
        .expect_send_message()
        .returning(|_, _| Ok(DiscordMessageId::new(20000)));

    // 解除時のモック設定
    mock_gateway.expect_get_channel().returning(|_| {
        use gbf_discord_bot_rs::types::discord::{ChannelData, ChannelKind, DiscordGuildId};
        Ok(ChannelData {
            id: DiscordChannelId::new(10000),
            guild_id: Some(DiscordGuildId::new(500)),
            name: "test".to_string(),
            kind: ChannelKind::Text,
            parent_id: None, // カテゴリ外のチャンネルとして扱う
            topic: None,
            position: Some(0),
        })
    });
    mock_gateway.expect_delete_channel().returning(|_| Ok(()));
    mock_gateway
        .expect_delete_message()
        .returning(|_, _| Ok(()));

    let guild_id = (AUTO_GUILD_ID + 500) as i64;
    let category_id = (guild_id + 10) as u64;
    let command_channel_id = (guild_id + 100) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // カテゴリ登録
    category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await
    .unwrap();

    // カテゴリ登録解除
    let result = category_setup_facade::unregister_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        command_channel_id,
    )
    .await;

    assert!(result.is_ok(), "カテゴリ登録解除に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

// =================================================
// change_days
// =================================================

/// 8-1: 正常系 - 日数増加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_days_increase() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();

    // 登録時のモック
    use gbf_discord_bot_rs::types::discord::{DiscordChannelId, DiscordMessageId};
    let mut call_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        call_count += 1;
        Ok(DiscordChannelId::new(10000 + call_count as u64))
    });
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));
    mock_gateway
        .expect_send_message()
        .returning(|_, _| Ok(DiscordMessageId::new(20000)));

    let guild_id = (AUTO_GUILD_ID + 600) as i64;
    let category_id = (guild_id + 10) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // カテゴリ登録（3日）
    category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await
    .unwrap();

    // 日数増加（3→5）
    let result =
        category_setup_facade::change_days(&mock_gateway, &app_state, guild_id as u64, 5).await;

    assert!(result.is_ok(), "日数増加に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

/// 8-2: 正常系 - 日数減少
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_days_decrease() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();

    // 登録時のモック
    use gbf_discord_bot_rs::types::discord::{DiscordChannelId, DiscordMessageId};
    let mut call_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        call_count += 1;
        Ok(DiscordChannelId::new(10000 + call_count as u64))
    });
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));
    mock_gateway
        .expect_send_message()
        .returning(|_, _| Ok(DiscordMessageId::new(20000)));

    // 削除時のモック
    mock_gateway.expect_delete_channel().returning(|_| Ok(()));
    mock_gateway
        .expect_delete_message()
        .returning(|_, _| Ok(()));

    let guild_id = (AUTO_GUILD_ID + 601) as i64;
    let category_id = (guild_id + 10) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // カテゴリ登録（5日）
    category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        5,
        None,
        None,
    )
    .await
    .unwrap();

    // 日数減少（5→3）
    let result =
        category_setup_facade::change_days(&mock_gateway, &app_state, guild_id as u64, 3).await;

    assert!(result.is_ok(), "日数減少に失敗: {:?}", result.err());

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

/// 8-3: 異常系 - 同じ日数への変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_days_same_value() {
    let app_state = Arc::new(create_test_app_state().await);
    let mock_gateway = MockTestGateway::new();

    let guild_id = (AUTO_GUILD_ID + 300) as i64;

    // 既存の自動募集データを削除
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    // 自動募集を事前登録（days=3）
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    // 同じ日数（3日）への変更を試みる
    let result =
        category_setup_facade::change_days(&mock_gateway, &app_state, guild_id as u64, 3).await;

    assert!(
        result.is_err(),
        "同じ日数への変更でエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("既に3日です"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );

    // クリーンアップ
    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

/// 8-4: 異常系 - 範囲外の日数
#[tokio::test]
async fn test_change_days_out_of_range() {
    if should_skip_for_missing_db_env() {
        return;
    }

    let app_state = Arc::new(create_test_app_state().await);
    let mock_gateway = MockTestGateway::new();

    let guild_id = (AUTO_GUILD_ID + 301) as u64;

    // days=1への変更を試みる（範囲外）
    let result = category_setup_facade::change_days(&mock_gateway, &app_state, guild_id, 1).await;

    assert!(result.is_err(), "days=1でエラーが返りませんでした");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("2〜7日の範囲"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );

    // days=8への変更を試みる（範囲外）
    let result = category_setup_facade::change_days(&mock_gateway, &app_state, guild_id, 8).await;

    assert!(result.is_err(), "days=8でエラーが返りませんでした");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("2〜7日の範囲"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}

/// 6-5: 異常系 - チャンネル作成失敗時のロールバック
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_category_channel_creation_failed_rollback() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = (AUTO_GUILD_ID + 900) as i64;
    let category_id = (guild_id + 10) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    mock_gateway.expect_create_channel().returning(|_, _| {
        Err(
            gbf_discord_bot_rs::errors::GatewayError::CreateChannelFailed(
                "create failed".to_string(),
            ),
        )
    });
    mock_gateway.expect_send_message().never();
    mock_gateway.expect_edit_channel().never();

    let result = category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await;

    assert!(
        result.is_err(),
        "チャンネル作成失敗でエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("ChannelCreationFailed"),
        "エラー種別が期待と異なります: {}",
        err_msg
    );

    let auto_recruitment = auto_recruitments::Entity::find()
        .filter(auto_recruitments::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap();
    assert!(
        auto_recruitment.is_none(),
        "ロールバック後にauto_recruitmentsが残っています"
    );
}

/// 7-2: 異常系 - カテゴリチャンネル内でのコマンド実行
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_category_in_category_channel_error() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = (AUTO_GUILD_ID + 901) as i64;
    let category_id = (guild_id + 10) as u64;
    let command_channel_id = (guild_id + 999) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
    setup_auto_recruitment_registered(&app_state, guild_id).await;

    mock_gateway.expect_get_channel().returning(move |_| {
        Ok(ChannelData {
            id: DiscordChannelId::new(command_channel_id),
            guild_id: Some(DiscordGuildId::new(guild_id as u64)),
            name: "カテゴリ内".to_string(),
            kind: ChannelKind::Text,
            parent_id: Some(DiscordChannelId::new(category_id)),
            topic: None,
            position: Some(0),
        })
    });
    mock_gateway.expect_delete_channel().never();
    mock_gateway.expect_delete_message().never();

    let result = category_setup_facade::unregister_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        command_channel_id,
    )
    .await;

    assert!(
        result.is_err(),
        "カテゴリ内チャンネル実行でエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("InCategoryChannelError"),
        "エラー種別が期待と異なります: {}",
        err_msg
    );

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}

/// 7-3: 異常系 - 未登録ギルドでの解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_category_not_registered() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();

    mock_gateway.expect_get_channel().never();
    mock_gateway.expect_delete_channel().never();
    mock_gateway.expect_delete_message().never();

    let result = category_setup_facade::unregister_category(
        &mock_gateway,
        &app_state,
        (AUTO_GUILD_ID + 902) as u64,
        (AUTO_GUILD_ID + 1902) as u64,
    )
    .await;

    assert!(
        result.is_err(),
        "未登録ギルド解除でエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("自動募集が登録されていません"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}

/// 7-4: 準正常系 - Discord削除失敗を含む解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_category_with_discord_delete_failures() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = (AUTO_GUILD_ID + 903) as i64;
    let category_id = (guild_id + 10) as u64;
    let command_channel_id = (guild_id + 2000) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    let mut create_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        create_count += 1;
        Ok(DiscordChannelId::new(50000 + create_count as u64))
    });
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));
    mock_gateway.expect_send_message().returning(|_, _| {
        Ok(gbf_discord_bot_rs::types::discord::DiscordMessageId::new(
            60000,
        ))
    });

    category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await
    .unwrap();

    mock_gateway.expect_get_channel().returning(move |_| {
        Ok(ChannelData {
            id: DiscordChannelId::new(command_channel_id),
            guild_id: Some(DiscordGuildId::new(guild_id as u64)),
            name: "カテゴリ外".to_string(),
            kind: ChannelKind::Text,
            parent_id: None,
            topic: None,
            position: Some(0),
        })
    });
    mock_gateway.expect_delete_channel().returning(|_| {
        Err(
            gbf_discord_bot_rs::errors::GatewayError::DeleteChannelFailed(
                "delete failed".to_string(),
            ),
        )
    });
    mock_gateway.expect_delete_message().returning(|_, _| {
        Err(
            gbf_discord_bot_rs::errors::GatewayError::DeleteMessageFailed(
                "delete message failed".to_string(),
            ),
        )
    });

    let result = category_setup_facade::unregister_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        command_channel_id,
    )
    .await;

    assert!(
        result.is_ok(),
        "Discord削除失敗を含む解除で失敗しました: {:?}",
        result.err()
    );

    let auto_recruitment = auto_recruitments::Entity::find()
        .filter(auto_recruitments::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap();
    assert!(
        auto_recruitment.is_none(),
        "解除後にauto_recruitmentsが残っています"
    );

    let date_channels = auto_recruitment_channels::Entity::find()
        .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert!(
        date_channels.is_empty(),
        "解除後にauto_recruitment_channelsが残っています"
    );

    let quest_messages = auto_recruitment_quest_messages::Entity::find()
        .filter(auto_recruitment_quest_messages::Column::GuildId.eq(guild_id))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert!(
        quest_messages.is_empty(),
        "解除後にauto_recruitment_quest_messagesが残っています"
    );
}

/// 8-5: 異常系 - 日数増加時のチャンネル作成失敗でロールバック
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_days_increase_channel_creation_failed_rollback() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = (AUTO_GUILD_ID + 904) as i64;
    let category_id = (guild_id + 10) as u64;

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;

    let mut create_count = 0;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        create_count += 1;
        Ok(DiscordChannelId::new(70000 + create_count as u64))
    });
    mock_gateway.expect_edit_channel().returning(|_, _| Ok(()));
    mock_gateway.expect_send_message().returning(|_, _| {
        Ok(gbf_discord_bot_rs::types::discord::DiscordMessageId::new(
            71000,
        ))
    });

    category_setup_facade::register_category(
        &mock_gateway,
        &app_state,
        guild_id as u64,
        category_id,
        3,
        None,
        None,
    )
    .await
    .unwrap();

    let before_days = auto_recruitments::Entity::find()
        .filter(auto_recruitments::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap()
        .days_range;
    let before_channels = auto_recruitment_channels::Entity::find()
        .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
        .all(app_state.guild_db())
        .await
        .unwrap()
        .len();

    let mut fail_once = true;
    mock_gateway.expect_create_channel().returning(move |_, _| {
        if fail_once {
            fail_once = false;
            Err(
                gbf_discord_bot_rs::errors::GatewayError::CreateChannelFailed(
                    "create failed in change".to_string(),
                ),
            )
        } else {
            Ok(DiscordChannelId::new(79999))
        }
    });

    let result =
        category_setup_facade::change_days(&mock_gateway, &app_state, guild_id as u64, 5).await;

    assert!(
        result.is_err(),
        "チャンネル作成失敗でchange_daysが成功してしまいました"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("ChannelCreationFailed"),
        "エラー種別が期待と異なります: {}",
        err_msg
    );

    let after = auto_recruitments::Entity::find()
        .filter(auto_recruitments::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        after.days_range, before_days,
        "ロールバック後にdays_rangeが変化しています"
    );

    let after_channels = auto_recruitment_channels::Entity::find()
        .filter(auto_recruitment_channels::Column::GuildId.eq(guild_id))
        .all(app_state.guild_db())
        .await
        .unwrap()
        .len();
    assert_eq!(
        after_channels, before_channels,
        "ロールバック後に日時チャンネル数が変化しています"
    );

    cleanup_auto_recruitment_data(app_state.guild_db(), guild_id, AUTO_USER_ID as i64).await;
}
