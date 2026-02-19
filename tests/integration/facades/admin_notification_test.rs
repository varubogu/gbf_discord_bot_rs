// 管理者通知サービス 結合テスト
//
// 対象: src/services/channel/admin_notification_service.rs

use gbf_discord_bot_rs::facades::channel::channel_management_facade::ChannelManagementFacade;
use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::models::entities::master::channel_types::GuildChannelType;
use gbf_discord_bot_rs::repository::database::SeaOrmGuildChannelRepository;
use gbf_discord_bot_rs::services::channel::AdminNotificationService;
use gbf_discord_bot_rs::types::discord::{DiscordMessageId, MessageContent};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;

use super::test_helper::{MockTestGateway, TEST_CHANNEL_ID, TEST_GUILD_ID, create_test_app_state};

/// テスト用ギルドID（管理者通知テスト専用）
const AN_TEST_GUILD_ID: i64 = TEST_GUILD_ID + 500;
/// テスト用チャンネルID（管理者通知チャンネル）
const AN_TEST_CHANNEL_ID: i64 = TEST_CHANNEL_ID + 500;

/// テスト後のクリーンアップ：ギルドチャンネルデータを削除
async fn cleanup_guild_channels(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    use gbf_discord_bot_rs::models::entities::guild_master::guild_channels;
    let _ = guild_channels::Entity::delete_many()
        .filter(guild_channels::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// テスト後のクリーンアップ：ギルドデータを削除
async fn cleanup_guild(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    use gbf_discord_bot_rs::models::entities::guild_master::guilds;
    let _ = guilds::Entity::delete_many()
        .filter(guilds::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// 全テストデータをクリーンアップ
async fn cleanup_all(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    cleanup_guild_channels(db, guild_id).await;
    cleanup_guild(db, guild_id).await;
}

/// テスト用ギルドと管理者通知チャンネルを事前登録
async fn setup_guild_with_admin_channel(
    app_state: &Arc<gbf_discord_bot_rs::types::AppState>,
    guild_id: i64,
    channel_id: i64,
) {
    let guild_facade = GuildManagementFacade::new(app_state.clone());
    guild_facade
        .register_new_guild(guild_id, "テスト用ギルド（管理者通知）")
        .await
        .unwrap();

    let ch_facade = ChannelManagementFacade::new(app_state.clone());
    ch_facade
        .register_channel(
            guild_id,
            "テスト用ギルド（管理者通知）".to_string(),
            GuildChannelType::AdminNotification.as_i32(),
            channel_id,
        )
        .await
        .unwrap();
}

// =================================================
// notify_admin
// =================================================

/// 1-1: 正常系 - 管理者通知チャンネルが設定されている場合、send_messageが呼ばれる
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_notify_admin_sends_message_when_channel_configured() {
    let app_state = Arc::new(create_test_app_state().await);
    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;
    setup_guild_with_admin_channel(&app_state, AN_TEST_GUILD_ID, AN_TEST_CHANNEL_ID).await;

    // Discordゲートウェイモック: send_messageが1回呼ばれることを期待
    let mut mock_gateway = MockTestGateway::new();
    mock_gateway
        .expect_send_message()
        .times(1)
        .returning(|_, _| Ok(DiscordMessageId::new(11111)));

    let service = AdminNotificationService::new(
        Arc::new(mock_gateway),
        SeaOrmGuildChannelRepository::new(),
    );

    let txn = app_state.guild_db().begin().await.unwrap();
    let result = service
        .notify_admin(&txn, AN_TEST_GUILD_ID, MessageContent::text("テストエラー通知"))
        .await;
    txn.commit().await.unwrap();

    assert!(
        result.is_ok(),
        "管理者通知送信に失敗: {:?}",
        result.err()
    );

    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;
}

/// 1-2: 正常系 - 管理者通知チャンネルが未設定の場合、send_messageは呼ばれずOkを返す
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_notify_admin_skips_when_channel_not_configured() {
    let app_state = Arc::new(create_test_app_state().await);
    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;

    // ギルドのみ登録（管理者通知チャンネルは登録しない）
    let guild_facade = GuildManagementFacade::new(app_state.clone());
    guild_facade
        .register_new_guild(AN_TEST_GUILD_ID, "テスト用ギルド（チャンネルなし）")
        .await
        .unwrap();

    // Discordゲートウェイモック: send_messageは呼ばれないことを期待
    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_send_message().times(0);

    let service = AdminNotificationService::new(
        Arc::new(mock_gateway),
        SeaOrmGuildChannelRepository::new(),
    );

    let txn = app_state.guild_db().begin().await.unwrap();
    let result = service
        .notify_admin(&txn, AN_TEST_GUILD_ID, MessageContent::text("テストエラー通知"))
        .await;
    txn.commit().await.unwrap();

    // チャンネル未設定でもOkが返ること
    assert!(
        result.is_ok(),
        "チャンネル未設定時にエラーが返った: {:?}",
        result.err()
    );

    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;
}

/// 1-3: 異常系 - Discord送信に失敗した場合、Errを返す
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_notify_admin_returns_err_on_gateway_failure() {
    use gbf_discord_bot_rs::errors::GatewayError;

    let app_state = Arc::new(create_test_app_state().await);
    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;
    setup_guild_with_admin_channel(&app_state, AN_TEST_GUILD_ID, AN_TEST_CHANNEL_ID).await;

    // Discordゲートウェイモック: send_messageがエラーを返す
    let mut mock_gateway = MockTestGateway::new();
    mock_gateway
        .expect_send_message()
        .times(1)
        .returning(|_, _| {
            Err(GatewayError::SendMessageFailed(
                "権限不足のためメッセージ送信に失敗".to_string(),
            ))
        });

    let service = AdminNotificationService::new(
        Arc::new(mock_gateway),
        SeaOrmGuildChannelRepository::new(),
    );

    let txn = app_state.guild_db().begin().await.unwrap();
    let result = service
        .notify_admin(&txn, AN_TEST_GUILD_ID, MessageContent::text("テストエラー通知"))
        .await;
    // エラーが返るためtxnはdropに任せる（rollback相当）

    assert!(result.is_err(), "送信失敗時にOkが返った");

    cleanup_all(app_state.guild_db(), AN_TEST_GUILD_ID).await;
}
