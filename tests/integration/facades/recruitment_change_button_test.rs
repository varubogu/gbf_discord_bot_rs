// 募集変更・ボタン操作ファサード 結合テスト
//
// 対象:
// - src/facades/recruitment/change.rs
// - src/facades/recruitment/button_handler.rs

use chrono::{Duration, Utc};
use gbf_discord_bot_rs::errors::GatewayError;
use gbf_discord_bot_rs::facades::recruitment::{button_handler, change};
use gbf_discord_bot_rs::models::entities::worker::{battle_recruitments, recruitment_participants};
use gbf_discord_bot_rs::types::discord::{
    ActionRowData, ChannelKind, ComponentData, DiscordChannelId, DiscordGuildId, DiscordMessageId,
    DiscordUserId, EmbedData, EmbedFieldData, MessageData, ReactionData, ReactionEmoji,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use super::test_helper::{
    MockTestGateway, TEST_CHANNEL_ID, TEST_GUILD_ID, TEST_MESSAGE_ID, TEST_USER_ID,
    create_test_app_state,
};

const CHANGE_GUILD_ID: i64 = TEST_GUILD_ID + 1200;
const CHANGE_CHANNEL_ID: i64 = TEST_CHANNEL_ID + 1200;
const CHANGE_MESSAGE_ID: i64 = TEST_MESSAGE_ID + 1200;

async fn create_test_recruitment(
    db: &sea_orm::DatabaseConnection,
    guild_id: i64,
    channel_id: i64,
    message_id: i64,
    quest_id: i32,
    battle_style_id: i32,
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
        quest_id: Set(quest_id),
        battle_style_id: Set(battle_style_id),
        quest_start_at: Set(quest_start_at),
        is_recruiting: Set(!is_canceled),
        is_canceled: Set(is_canceled),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    model.insert(db).await.unwrap()
}

async fn cleanup_recruitment_with_participants(
    db: &sea_orm::DatabaseConnection,
    recruitment_id: i32,
) {
    let _ = recruitment_participants::Entity::delete_many()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment_id))
        .exec(db)
        .await;
    let _ = battle_recruitments::Entity::delete_by_id(recruitment_id)
        .exec(db)
        .await;
}

async fn add_test_participant(
    db: &sea_orm::DatabaseConnection,
    recruitment_id: i32,
    user_id: u64,
    element_id: Option<i32>,
) {
    recruitment_participants::ActiveModel {
        recruitment_id: Set(recruitment_id),
        user_id: Set(user_id as i64),
        element_id: Set(element_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();
}

fn create_message_for_change(channel_id: u64, message_id: u64) -> MessageData {
    MessageData {
        id: DiscordMessageId::new(message_id),
        channel_id: DiscordChannelId::new(channel_id),
        author_id: gbf_discord_bot_rs::types::discord::DiscordUserId::new(TEST_USER_ID),
        content: "募集本文".to_string(),
        embeds: vec![EmbedData {
            title: Some("参加者一覧".to_string()),
            description: Some("初期".to_string()),
            color: Some(0x0099ff),
            fields: vec![EmbedFieldData {
                name: "f".to_string(),
                value: "v".to_string(),
                inline: false,
            }],
            footer_text: Some("参加者数: 0人".to_string()),
        }],
        components: vec![ActionRowData {
            components: vec![ComponentData::Unknown],
        }],
        reactions: vec![],
        pinned: false,
        referenced_message_id: None,
    }
}

fn setup_common_gateway_for_change(mock_gateway: &mut MockTestGateway) {
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));
    mock_gateway
        .expect_send_reply()
        .returning(|_, _, _, _| Ok(DiscordMessageId::new(987650)));
}

fn setup_common_gateway_for_button(
    mock_gateway: &mut MockTestGateway,
    channel_id: u64,
    message_id: u64,
) {
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
    mock_gateway.expect_get_message().returning(move |_, _| {
        Ok(MessageData {
            id: DiscordMessageId::new(message_id),
            channel_id: DiscordChannelId::new(channel_id),
            author_id: gbf_discord_bot_rs::types::discord::DiscordUserId::new(TEST_USER_ID),
            content: "募集本文".to_string(),
            embeds: vec![EmbedData {
                title: Some("参加者一覧".to_string()),
                description: Some("初期".to_string()),
                color: Some(0x0099ff),
                fields: vec![],
                footer_text: Some("参加者数: 0人".to_string()),
            }],
            components: vec![],
            reactions: vec![ReactionData {
                emoji: ReactionEmoji::Unicode("✅".to_string()),
                count: 1,
            }],
            pinned: false,
            referenced_message_id: None,
        })
    });
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));
    mock_gateway
        .expect_send_reply()
        .times(0..)
        .returning(|_, _, _, _| Ok(DiscordMessageId::new(999001)));
    mock_gateway
        .expect_get_channel()
        .times(0..)
        .returning(|channel_id| {
            Ok(gbf_discord_bot_rs::types::discord::ChannelData {
                id: channel_id,
                guild_id: Some(DiscordGuildId::new(CHANGE_GUILD_ID as u64)),
                name: "test".to_string(),
                kind: ChannelKind::Text,
                parent_id: None,
                topic: None,
                position: Some(0),
            })
        });
}

/// 6-1: 正常系 - クエスト変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_change_quest() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_change(&mut mock_gateway);

    let guild_id = CHANGE_GUILD_ID + 1;
    let channel_id = CHANGE_CHANNEL_ID + 1;
    let message_id = CHANGE_MESSAGE_ID + 1;
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    let message = create_message_for_change(channel_id as u64, message_id as u64);

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        Some("ルシファーHL"),
        None,
        None,
    )
    .await;
    assert!(result.is_ok(), "クエスト変更に失敗: {:?}", result.err());

    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_ne!(
        updated.quest_id, recruitment.quest_id,
        "quest_idが変更されていません"
    );

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 6-2: 正常系 - 開催日時変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_change_event_date() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_change(&mut mock_gateway);

    let guild_id = CHANGE_GUILD_ID + 2;
    let channel_id = CHANGE_CHANNEL_ID + 2;
    let message_id = CHANGE_MESSAGE_ID + 2;
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    let message = create_message_for_change(channel_id as u64, message_id as u64);
    let new_date = Utc::now() + Duration::hours(96);

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        None,
        Some(new_date),
        None,
    )
    .await;
    assert!(result.is_ok(), "開催日時変更に失敗: {:?}", result.err());

    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    let diff = (updated.quest_start_at - new_date).num_minutes().abs();
    assert!(diff <= 1, "quest_start_atが更新されていません");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 6-3: 正常系 - 攻略方法変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_change_battle_style() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_change(&mut mock_gateway);

    let guild_id = CHANGE_GUILD_ID + 3;
    let channel_id = CHANGE_CHANNEL_ID + 3;
    let message_id = CHANGE_MESSAGE_ID + 3;
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    let message = create_message_for_change(channel_id as u64, message_id as u64);

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        None,
        None,
        Some(2),
    )
    .await;
    assert!(result.is_ok(), "攻略方法変更に失敗: {:?}", result.err());

    let updated = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        updated.battle_style_id, 2,
        "battle_style_idが更新されていません"
    );

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 6-4: 異常系 - 存在しない募集の変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_change(&mut mock_gateway);

    let message = create_message_for_change(
        (CHANGE_CHANNEL_ID + 4) as u64,
        (CHANGE_MESSAGE_ID + 4) as u64,
    );
    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        (CHANGE_GUILD_ID + 4) as u64,
        &message,
        Some("ルシファーHL"),
        None,
        None,
    )
    .await;

    assert!(result.is_err(), "存在しない募集でエラーが返りませんでした");
}

/// 6-5: 異常系 - Gateway編集失敗時のロールバック
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_edit_failed_rollback() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Err(GatewayError::EditMessageFailed("edit failed".to_string())));
    mock_gateway.expect_send_reply().never();

    let guild_id = CHANGE_GUILD_ID + 5;
    let channel_id = CHANGE_CHANNEL_ID + 5;
    let message_id = CHANGE_MESSAGE_ID + 5;
    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    let message = create_message_for_change(channel_id as u64, message_id as u64);
    let old_quest_id = recruitment.quest_id;

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        Some("ルシファーHL"),
        None,
        None,
    )
    .await;
    assert!(result.is_err(), "Gateway編集失敗でエラーが返りませんでした");

    let unchanged = battle_recruitments::Entity::find_by_id(recruitment.id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        unchanged.quest_id, old_quest_id,
        "ロールバックされずquest_idが更新されています"
    );

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 6-6: 正常系 - v2募集の変更通知でDB+リアクションを合算し重複除去する
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_v2_notification_mentions_union_dedup() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CHANGE_GUILD_ID + 6;
    let channel_id = CHANGE_CHANNEL_ID + 6;
    let message_id = CHANGE_MESSAGE_ID + 6;
    let duplicated_user_id: u64 = 555_555_555;
    let reaction_only_user_id: u64 = 666_666_666;

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    add_test_participant(
        app_state.guild_db(),
        recruitment.id,
        duplicated_user_id,
        Some(1),
    )
    .await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
    mock_gateway
        .expect_get_reaction_users()
        .returning(move |_, _, _, _| {
            Ok(vec![
                DiscordUserId::new(duplicated_user_id),
                DiscordUserId::new(reaction_only_user_id),
            ])
        });
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));

    let duplicated_mention = format!("<@{duplicated_user_id}>");
    let reaction_only_mention = format!("<@{reaction_only_user_id}>");
    mock_gateway
        .expect_send_reply()
        .withf(move |_, _, content, _| {
            content.text.as_ref().is_some_and(|text| {
                text.contains(&duplicated_mention)
                    && text.contains(&reaction_only_mention)
                    && text.matches(&duplicated_mention).count() == 1
            })
        })
        .returning(|_, _, _, _| Ok(DiscordMessageId::new(987651)));

    let mut message = create_message_for_change(channel_id as u64, message_id as u64);
    message.reactions = vec![ReactionData {
        emoji: ReactionEmoji::unicode("⚔"),
        count: 2,
    }];

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        Some("ルシファーHL"),
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "v2通知の合算に失敗: {:?}", result.err());
    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 6-7: 正常系 - v1募集の変更通知でDB+リアクションを合算する
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_change_recruitment_information_v1_notification_mentions_union() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = CHANGE_GUILD_ID + 7;
    let channel_id = CHANGE_CHANNEL_ID + 7;
    let message_id = CHANGE_MESSAGE_ID + 7;
    let db_user_id: u64 = 777_777_777;
    let reaction_user_id: u64 = 888_888_888;

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    add_test_participant(app_state.guild_db(), recruitment.id, db_user_id, None).await;

    let mut mock_gateway = MockTestGateway::new();
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
    mock_gateway
        .expect_get_reaction_users()
        .returning(move |_, _, _, _| Ok(vec![DiscordUserId::new(reaction_user_id)]));
    mock_gateway
        .expect_edit_message()
        .returning(|_, _, _| Ok(()));

    let db_mention = format!("<@{db_user_id}>");
    let reaction_mention = format!("<@{reaction_user_id}>");
    mock_gateway
        .expect_send_reply()
        .withf(move |_, _, content, _| {
            content
                .text
                .as_ref()
                .is_some_and(|text| text.contains(&db_mention) && text.contains(&reaction_mention))
        })
        .returning(|_, _, _, _| Ok(DiscordMessageId::new(987652)));

    let mut message = create_message_for_change(channel_id as u64, message_id as u64);
    message.components = vec![]; // v1募集として扱う
    message.reactions = vec![ReactionData {
        emoji: ReactionEmoji::unicode("⚔"),
        count: 1,
    }];

    let result = change::change_recruitment_information_internal(
        &app_state,
        &mock_gateway,
        guild_id as u64,
        &message,
        Some("ルシファーHL"),
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "v1通知の合算に失敗: {:?}", result.err());
    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-1: 正常系 - 参加ボタン押下
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_join() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 10;
    let channel_id = CHANGE_CHANNEL_ID + 10;
    let message_id = CHANGE_MESSAGE_ID + 10;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        "recruit_join",
    )
    .await;
    assert!(result.is_ok(), "参加ボタン処理に失敗: {:?}", result.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert_eq!(participants.len(), 1, "参加者が登録されていません");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-2: 正常系 - 属性指定参加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_join_element() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 11;
    let channel_id = CHANGE_CHANNEL_ID + 11;
    let message_id = CHANGE_MESSAGE_ID + 11;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        "recruit_join_1",
    )
    .await;
    assert!(result.is_ok(), "属性指定参加に失敗: {:?}", result.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].element_id, Some(1));

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-3: 正常系 - 全属性参加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_join_all_elements() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 12;
    let channel_id = CHANGE_CHANNEL_ID + 12;
    let message_id = CHANGE_MESSAGE_ID + 12;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        "recruit_join_0",
    )
    .await;
    assert!(result.is_ok(), "全属性参加に失敗: {:?}", result.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].element_id, None);

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-4: 正常系 - 退出ボタン押下
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_leave_all() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 13;
    let channel_id = CHANGE_CHANNEL_ID + 13;
    let message_id = CHANGE_MESSAGE_ID + 13;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;
    recruitment_participants::ActiveModel {
        recruitment_id: Set(recruitment.id),
        user_id: Set(TEST_USER_ID as i64),
        element_id: Set(Some(1)),
        ..Default::default()
    }
    .insert(app_state.guild_db())
    .await
    .unwrap();

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        "recruit_leave_all",
    )
    .await;
    assert!(result.is_ok(), "退出処理に失敗: {:?}", result.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert!(participants.is_empty(), "退出後に参加者が残っています");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-5: 異常系 - キャンセル済み募集への操作
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_cancelled_error() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_button(
        &mut mock_gateway,
        (CHANGE_CHANNEL_ID + 14) as u64,
        (CHANGE_MESSAGE_ID + 14) as u64,
    );

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        CHANGE_GUILD_ID + 14,
        CHANGE_CHANNEL_ID + 14,
        CHANGE_MESSAGE_ID + 14,
        1,
        1,
        true,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new((CHANGE_GUILD_ID + 14) as u64),
        DiscordChannelId::new((CHANGE_CHANNEL_ID + 14) as u64),
        DiscordMessageId::new((CHANGE_MESSAGE_ID + 14) as u64),
        TEST_USER_ID,
        "recruit_join",
    )
    .await;
    assert!(
        result.is_err(),
        "キャンセル済み募集でエラーが返りませんでした"
    );

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 7-6: 異常系 - 期限切れ募集への操作
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_button_expired_error() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_common_gateway_for_button(
        &mut mock_gateway,
        (CHANGE_CHANNEL_ID + 15) as u64,
        (CHANGE_MESSAGE_ID + 15) as u64,
    );

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        CHANGE_GUILD_ID + 15,
        CHANGE_CHANNEL_ID + 15,
        CHANGE_MESSAGE_ID + 15,
        1,
        1,
        false,
        -1,
    )
    .await;

    let result = button_handler::handle_recruitment_button(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new((CHANGE_GUILD_ID + 15) as u64),
        DiscordChannelId::new((CHANGE_CHANNEL_ID + 15) as u64),
        DiscordMessageId::new((CHANGE_MESSAGE_ID + 15) as u64),
        TEST_USER_ID,
        "recruit_join",
    )
    .await;
    assert!(result.is_err(), "期限切れ募集でエラーが返りませんでした");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 8-1: 正常系 - 複数属性選択
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_select_menu_multi_elements() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 20;
    let channel_id = CHANGE_CHANNEL_ID + 20;
    let message_id = CHANGE_MESSAGE_ID + 20;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_select_menu(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        vec![1, 3],
    )
    .await;
    assert!(result.is_ok(), "複数属性選択に失敗: {:?}", result.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    assert_eq!(participants.len(), 2, "複数属性が登録されていません");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 8-2: 正常系 - 既存参加の上書き
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_select_menu_overwrite_like_behavior() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 21;
    let channel_id = CHANGE_CHANNEL_ID + 21;
    let message_id = CHANGE_MESSAGE_ID + 21;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        false,
        24,
    )
    .await;

    let first = button_handler::handle_recruitment_select_menu(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        vec![1],
    )
    .await;
    assert!(first.is_ok(), "初回選択に失敗: {:?}", first.err());

    let second = button_handler::handle_recruitment_select_menu(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        vec![1, 2],
    )
    .await;
    assert!(second.is_ok(), "再選択に失敗: {:?}", second.err());

    let participants = recruitment_participants::Entity::find()
        .filter(recruitment_participants::Column::RecruitmentId.eq(recruitment.id))
        .filter(recruitment_participants::Column::UserId.eq(TEST_USER_ID as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();
    let element_ids: Vec<i32> = participants.iter().filter_map(|p| p.element_id).collect();
    assert!(element_ids.contains(&2), "新しい属性が反映されていません");

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}

/// 8-3: 異常系 - キャンセル済み募集への操作
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_handle_recruitment_select_menu_cancelled_error() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    let guild_id = CHANGE_GUILD_ID + 22;
    let channel_id = CHANGE_CHANNEL_ID + 22;
    let message_id = CHANGE_MESSAGE_ID + 22;
    setup_common_gateway_for_button(&mut mock_gateway, channel_id as u64, message_id as u64);

    let recruitment = create_test_recruitment(
        app_state.guild_db(),
        guild_id,
        channel_id,
        message_id,
        1,
        1,
        true,
        24,
    )
    .await;

    let result = button_handler::handle_recruitment_select_menu(
        &mock_gateway,
        &app_state,
        DiscordGuildId::new(guild_id as u64),
        DiscordChannelId::new(channel_id as u64),
        DiscordMessageId::new(message_id as u64),
        TEST_USER_ID,
        vec![1, 3],
    )
    .await;
    assert!(
        result.is_err(),
        "キャンセル済み募集へのセレクト操作でエラーが返りませんでした"
    );

    cleanup_recruitment_with_participants(app_state.guild_db(), recruitment.id).await;
}
