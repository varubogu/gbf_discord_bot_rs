// 募集キャンセルファサード 結合テスト
//
// 対象: src/facades/recruitment/cancel.rs

use chrono::{Duration, Utc};
use gbf_discord_bot_rs::facades::recruitment::cancel;
use gbf_discord_bot_rs::models::entities::worker::battle_recruitments;
use gbf_discord_bot_rs::types::discord::DiscordMessageId;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::sync::Arc;

use super::test_helper::{
    MockTestGateway, TEST_CHANNEL_ID, TEST_GUILD_ID, TEST_MESSAGE_ID, create_test_app_state,
};

/// テスト用ギルドID（キャンセルテスト専用）
const CANCEL_GUILD_ID: i64 = TEST_GUILD_ID + 500;
const CANCEL_CHANNEL_ID: i64 = TEST_CHANNEL_ID + 500;
const CANCEL_MESSAGE_ID: i64 = TEST_MESSAGE_ID + 500;

/// テスト用募集レコードを作成
async fn create_test_recruitment(
    db: &sea_orm::DatabaseConnection,
    guild_id: i64,
    channel_id: i64,
    message_id: i64,
    is_canceled: bool,
    quest_start_at_offset_hours: i64,
) -> battle_recruitments::Model {
    let quest_start_at = if quest_start_at_offset_hours >= 0 {
        Utc::now() + Duration::hours(quest_start_at_offset_hours)
    } else {
        Utc::now() - Duration::hours(-quest_start_at_offset_hours)
    };

    let model = battle_recruitments::ActiveModel {
        guild_id: Set(guild_id),
        channel_id: Set(channel_id),
        message_id: Set(message_id),
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(quest_start_at),
        is_recruiting: Set(!is_canceled),
        is_canceled: Set(is_canceled),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    model.insert(db).await.unwrap()
}

/// テスト用募集レコードを削除
async fn cleanup_recruitment(db: &sea_orm::DatabaseConnection, recruitment_id: i32) {
    let _ = battle_recruitments::Entity::delete_by_id(recruitment_id)
        .exec(db)
        .await;
}

// =================================================
// cancel_on_message_deleted
// =================================================

/// 5-2: 正常系 - 募集メッセージでない場合
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_cancel_on_delete_not_recruitment_message() {
    let app_state = Arc::new(create_test_app_state().await);

    let mut mock_gateway = MockTestGateway::new();
    // send_messageが呼ばれないことを確認（募集でないため通知なし）
    mock_gateway.expect_send_message().never();

    let result = cancel::cancel_on_message_deleted(
        &mock_gateway,
        CANCEL_GUILD_ID as u64,
        CANCEL_CHANNEL_ID as u64,
        99999999_u64, // 存在しないメッセージID
        &app_state,
    )
    .await;

    assert!(result.is_ok());
    match result.unwrap() {
        gbf_discord_bot_rs::types::CancelOnDeleteResult::NotRecruitmentMessage => {}
        other => panic!(
            "NotRecruitmentMessageが期待されましたが {:?} が返りました",
            other
        ),
    }
}

/// 5-3: 正常系 - 既にキャンセル済みの場合
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_cancel_on_delete_already_cancelled() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 1;
    let channel_id = CANCEL_CHANNEL_ID + 1;
    let message_id = CANCEL_MESSAGE_ID + 1;

    // キャンセル済み募集を作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        true, // is_canceled = true
        24,   // 24時間後（未来）
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_send_message().never();

    let result = cancel::cancel_on_message_deleted(
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        &app_state,
    )
    .await;

    assert!(result.is_ok());
    match result.unwrap() {
        gbf_discord_bot_rs::types::CancelOnDeleteResult::AlreadyCancelled => {}
        other => panic!(
            "AlreadyCancelledが期待されましたが {:?} が返りました",
            other
        ),
    }

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 5-4: 正常系 - 開催日時を過ぎている場合
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_cancel_on_delete_event_date_passed() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 2;
    let channel_id = CANCEL_CHANNEL_ID + 2;
    let message_id = CANCEL_MESSAGE_ID + 2;

    // 開催日時が過去の募集を作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false, // is_canceled = false
        -1,    // 1時間前（過去）
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_send_message().never();

    let result = cancel::cancel_on_message_deleted(
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        &app_state,
    )
    .await;

    assert!(result.is_ok());
    match result.unwrap() {
        gbf_discord_bot_rs::types::CancelOnDeleteResult::EventDatePassed => {}
        other => panic!("EventDatePassedが期待されましたが {:?} が返りました", other),
    }

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 5-1: 正常系 - 募集メッセージ削除時のキャンセル
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_cancel_on_delete_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 3;
    let channel_id = CANCEL_CHANNEL_ID + 3;
    let message_id = CANCEL_MESSAGE_ID + 3;

    // 未来の募集を作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false, // is_canceled = false
        24,    // 24時間後（未来）
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();

    // send_messageが呼ばれることを期待（キャンセル通知送信）
    mock_gateway
        .expect_send_message()
        .returning(|_, _| Ok(DiscordMessageId::new(99999)));

    let result = cancel::cancel_on_message_deleted(
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        &app_state,
    )
    .await;

    assert!(result.is_ok(), "キャンセル処理に失敗: {:?}", result.err());
    match result.unwrap() {
        gbf_discord_bot_rs::types::CancelOnDeleteResult::Cancelled => {}
        other => panic!("Cancelledが期待されましたが {:?} が返りました", other),
    }

    // DBでキャンセル済みになっていることを確認
    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert!(updated.is_canceled, "DBでis_canceledがtrueになっていません");

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

// =================================================
// execute_cancel
// =================================================

/// 4-2: 異常系 - 開催日時を過ぎた募集のキャンセル
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_execute_cancel_event_passed() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 4;
    let channel_id = CANCEL_CHANNEL_ID + 4;
    let message_id = CANCEL_MESSAGE_ID + 4;

    // 開催日時が過去の募集を作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false,
        -1, // 1時間前（過去）
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    // get_messageが呼ばれないはず（開催日時チェックで弾かれるため）
    mock_gateway.expect_get_message().never();

    let result = cancel::execute_cancel(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        Some("ja"),
    )
    .await;

    assert!(
        result.is_err(),
        "開催日時過ぎの募集でエラーが返りませんでした"
    );

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 4-3: 異常系 - 存在しない募集のキャンセル
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_execute_cancel_not_found() {
    let app_state = Arc::new(create_test_app_state().await);

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().never();

    let result = cancel::execute_cancel(
        &app_state,
        &mock_gateway,
        CANCEL_GUILD_ID as u64 + 99,
        CANCEL_CHANNEL_ID as u64 + 99,
        99999999_u64,
        Some("ja"),
    )
    .await;

    assert!(result.is_err(), "存在しない募集でエラーが返りませんでした");
}
