// 募集キャンセルファサード 結合テスト
//
// 対象: src/facades/recruitment/cancel.rs

use chrono::{Duration, Utc};
use gbf_discord_bot_rs::errors::GatewayError;
use gbf_discord_bot_rs::facades::recruitment::cancel;
use gbf_discord_bot_rs::models::entities::worker::battle_recruitments;
use gbf_discord_bot_rs::types::discord::{
    DiscordChannelId, DiscordGuildId, DiscordMessageId, DiscordUserId, ReactionData, ReactionEmoji,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::sync::Arc;

use super::test_helper::{
    MockTestGateway, TEST_CHANNEL_ID, TEST_GUILD_ID, TEST_MESSAGE_ID, create_test_app_state,
    create_test_message_data,
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

/// 4-1: 正常系 - 募集キャンセル
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_execute_cancel_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 30;
    let channel_id = CANCEL_CHANNEL_ID + 30;
    let message_id = CANCEL_MESSAGE_ID + 30;

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false,
        24,
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456,
            "募集本文",
        ))
    });
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));
    mock_gateway
        .expect_send_reply()
        .returning(|_, _, _, _| Ok(DiscordMessageId::new(123450)));

    let result = cancel::execute_cancel(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        Some("ja"),
    )
    .await;

    assert!(result.is_ok(), "募集キャンセルに失敗: {:?}", result.err());

    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert!(updated.is_canceled, "is_canceledが更新されていません");
    assert!(
        updated.recruit_end_message_id.is_some(),
        "キャンセル通知メッセージIDが保存されていません"
    );

    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

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

/// 4-4: 異常系 - Gateway編集失敗時のロールバック
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_execute_cancel_edit_message_failed_rollback() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 31;
    let channel_id = CANCEL_CHANNEL_ID + 31;
    let message_id = CANCEL_MESSAGE_ID + 31;

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false,
        24,
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456,
            "募集本文",
        ))
    });
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Err(GatewayError::EditMessageFailed("edit failed".to_string())));
    mock_gateway.expect_send_reply().never();

    let result = cancel::execute_cancel(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        channel_id as u64,
        message_id as u64,
        Some("ja"),
    )
    .await;

    assert!(result.is_err(), "Gateway編集失敗でエラーが返りませんでした");

    let unchanged = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert!(
        !unchanged.is_canceled,
        "ロールバックされずis_canceledが更新されています"
    );
    assert!(
        unchanged.recruit_end_message_id.is_none(),
        "ロールバックされずrecruit_end_message_idが更新されています"
    );

    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

// =================================================
// can_cancel
// =================================================

/// 3-1: 正常系 - キャンセル可能な募集
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_available() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 10;
    let channel_id = CANCEL_CHANNEL_ID + 10;
    let message_id = CANCEL_MESSAGE_ID + 10;

    // 募集中・未キャンセル・開催日時前のレコードを作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false, // is_canceled = false
        24,    // 24時間後
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456, // author_id
            "test content",
        ))
    });

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::Success => {
            // キャンセル可能が返る
        }
        other => panic!("Successが期待されましたが {:?} が返りました", other),
    }

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 3-2: 正常系 - キャンセル済みの募集
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_already_cancelled() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 11;
    let channel_id = CANCEL_CHANNEL_ID + 11;
    let message_id = CANCEL_MESSAGE_ID + 11;

    // キャンセル済みのレコードを作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        true, // is_canceled = true
        24,
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456, // author_id
            "test content",
        ))
    });

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::AlreadyCancelled => {
            // キャンセル済みが返る
        }
        other => panic!(
            "AlreadyCancelledが期待されましたが {:?} が返りました",
            other
        ),
    }

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 3-3: 正常系 - DBに募集あり + Discordメッセージ削除済み
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_message_deleted() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 12;
    let channel_id = CANCEL_CHANNEL_ID + 12;
    let message_id = CANCEL_MESSAGE_ID + 12;

    // 募集中・未キャンセル・開催日時前のレコードを作成
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false,
        24,
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(|_, _| {
        Err(GatewayError::GetMessageFailed(
            "Message not found".to_string(),
        ))
    }); // Discord上ではメッセージ削除済み

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::MessageDeleted => {
            // DBにはあるがDiscordメッセージは削除済み
        }
        other => panic!("MessageDeletedが期待されましたが {:?} が返りました", other),
    }

    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 3-4: 正常系 - DBに募集なし + Discordメッセージは存在
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_not_recruit_message() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 13;
    let channel_id = CANCEL_CHANNEL_ID + 13;
    let message_id = CANCEL_MESSAGE_ID + 13;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456, // author_id
            "通常メッセージ",
        ))
    });

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::NotRecruitMessage => {
            // 募集メッセージではない
        }
        other => panic!(
            "NotRecruitMessageが期待されましたが {:?} が返りました",
            other
        ),
    }
}

/// 3-5: 正常系 - 開催日時を過ぎた募集
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_event_date_passed() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 14;
    let channel_id = CANCEL_CHANNEL_ID + 14;
    let message_id = CANCEL_MESSAGE_ID + 14;

    // 開催日時が過去のレコードを作成
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
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(create_test_message_data(
            message_id as u64,
            channel_id as u64,
            123456, // author_id
            "過去募集メッセージ",
        ))
    });

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::EventDatePassed => {
            // 開催日時超過のためキャンセル不可
        }
        other => panic!("EventDatePassedが期待されましたが {:?} が返りました", other),
    }

    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}

/// 3-6: 正常系 - 存在しないメッセージの募集
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_can_cancel_not_found() {
    let app_state = Arc::new(create_test_app_state().await);

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_message().returning(|_, _| {
        Err(GatewayError::GetMessageFailed(
            "Message not found".to_string(),
        ))
    }); // メッセージが見つからない

    let result = cancel::can_cancel(
        &app_state,
        &mock_gateway,
        DiscordGuildId::new(CANCEL_GUILD_ID as u64 + 99),
        DiscordChannelId::new(CANCEL_CHANNEL_ID as u64 + 99),
        DiscordMessageId::new(99999999_u64),
    )
    .await;

    assert!(
        result.is_ok(),
        "キャンセル可否確認に失敗: {:?}",
        result.err()
    );

    match result.unwrap() {
        gbf_discord_bot_rs::types::CanCancelResult::NotFound => {
            // 募集が見つからないが返る
        }
        other => panic!("NotFoundが期待されましたが {:?} が返りました", other),
    }
}

// =================================================
// bot除外（リアクション参加者通知）
// =================================================

/// 4-5: 正常系 - リアクションした参加者がキャンセル通知に含まれる（botは除外済み）
///
/// Gatewayの `get_reaction_users` はbotユーザーを除外して返す。
/// このテストではbotを除外した後の参加者IDのみをモックが返すことで、
/// キャンセル通知に人間の参加者メンションが正しく含まれることを検証する。
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_execute_cancel_participants_notified_bot_excluded() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CANCEL_GUILD_ID + 40;
    let channel_id = CANCEL_CHANNEL_ID + 40;
    let message_id = CANCEL_MESSAGE_ID + 40;

    // 人間の参加者ユーザーID（botは含まない）
    let human_user_id: u64 = 111111111;

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        false,
        24,
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();

    // リアクション付きメッセージを返す
    mock_gateway.expect_get_message().returning(move |_, _| {
        let mut msg =
            create_test_message_data(message_id as u64, channel_id as u64, 123456, "募集本文");
        msg.reactions = vec![ReactionData {
            emoji: ReactionEmoji::unicode("⚔"),
            count: 1,
        }];
        Ok(msg)
    });

    // Gatewayはbotを除外した後の参加者IDのみ返す（bot除外はGateway実装で行われる）
    mock_gateway
        .expect_get_reaction_users()
        .returning(move |_, _, _, _| Ok(vec![DiscordUserId::new(human_user_id)]));

    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));

    // キャンセル通知に人間ユーザーのメンションが含まれることを検証
    let expected_mention = format!("<@{human_user_id}>");
    mock_gateway
        .expect_send_reply()
        .withf(move |_, _, content, _| {
            content
                .text
                .as_ref()
                .map(|t| t.contains(&format!("<@{human_user_id}>")))
                .unwrap_or(false)
        })
        .returning(move |_, _, _, _| Ok(DiscordMessageId::new(123450)));

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
        result.is_ok(),
        "参加者ありのキャンセルに失敗: {:?}",
        result.err()
    );

    // DBでキャンセル済みになっていることを確認
    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert!(updated.is_canceled, "DBでis_canceledがtrueになっていません");

    // expected_mentionが参照されていることを明示（コンパイラの未使用警告を抑制）
    let _ = expected_mention;

    cleanup_recruitment(app_state.guild_db(), recruitment.id).await;
}
