// 新規募集ファサード 結合テスト
//
// 対象: src/facades/recruitment/new_recruit.rs

use gbf_discord_bot_rs::facades::recruitment::new_recruit;
use gbf_discord_bot_rs::infrastructure::database::session::set_current_guild_id;
use gbf_discord_bot_rs::models::entities::worker::battle_recruitments;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;

use super::test_helper::{
    MockTestGateway, TEST_CHANNEL_ID, TEST_GUILD_ID, TEST_USER_ID, create_test_app_state,
};

/// テスト用ID（新規募集テスト専用）
const NEW_GUILD_ID: u64 = (TEST_GUILD_ID + 600) as u64;
const NEW_CHANNEL_ID: u64 = (TEST_CHANNEL_ID + 600) as u64;

/// ギルド単位で募集関連データを削除
async fn cleanup_recruitments_by_guild(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    use gbf_discord_bot_rs::models::entities::worker::{
        battle_recruitment_dismissals, notification_rel_battle_recruitments,
    };

    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, guild_id).await.unwrap();

    let recruitments = battle_recruitments::Entity::find()
        .filter(battle_recruitments::Column::GuildId.eq(guild_id))
        .all(&txn)
        .await
        .unwrap();
    let recruitment_ids: Vec<i32> = recruitments.iter().map(|r| r.id).collect();

    if !recruitment_ids.is_empty() {
        let _ = battle_recruitment_dismissals::Entity::delete_many()
            .filter(
                battle_recruitment_dismissals::Column::RecruitmentId.is_in(recruitment_ids.clone()),
            )
            .exec(&txn)
            .await;

        let _ = notification_rel_battle_recruitments::Entity::delete_many()
            .filter(
                notification_rel_battle_recruitments::Column::RecruitId
                    .is_in(recruitment_ids.clone()),
            )
            .exec(&txn)
            .await;

        let _ = battle_recruitments::Entity::delete_many()
            .filter(battle_recruitments::Column::Id.is_in(recruitment_ids))
            .exec(&txn)
            .await;
    }

    txn.commit().await.unwrap();
}

/// 募集レコードを削除
async fn cleanup_recruitment(db: &sea_orm::DatabaseConnection, guild_id: i64, recruitment_id: i32) {
    // 関連する通知関連テーブルの削除
    use gbf_discord_bot_rs::models::entities::worker::notification_rel_battle_recruitments;

    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, guild_id).await.unwrap();

    let _ = notification_rel_battle_recruitments::Entity::delete_many()
        .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruitment_id))
        .exec(&txn)
        .await;

    let _ = battle_recruitments::Entity::delete_by_id(recruitment_id)
        .exec(&txn)
        .await;

    txn.commit().await.unwrap();
}

fn setup_new_recruitment_gateway(mock_gateway: &mut MockTestGateway) {
    mock_gateway.expect_get_emojis().returning(|_| Ok(vec![]));
}

// =================================================
// update_message_id
// =================================================

/// 2-1: 正常系 - message_id更新
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_update_message_id_success() {
    let app_state = Arc::new(create_test_app_state().await);

    // テスト用募集を直接作成
    use chrono::{Duration, Utc};
    use sea_orm::{ActiveModelTrait, Set};

    let model = battle_recruitments::ActiveModel {
        guild_id: Set(NEW_GUILD_ID as i64),
        channel_id: Set(NEW_CHANNEL_ID as i64),
        message_id: Set(0), // 仮のmessage_id
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(Utc::now() + Duration::hours(24)),
        is_recruiting: Set(true),
        is_canceled: Set(false),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    let db = app_state.guild_db();
    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, NEW_GUILD_ID as i64)
        .await
        .unwrap();
    let inserted = model.insert(&txn).await.unwrap();
    txn.commit().await.unwrap();

    // message_id更新
    let new_message_id = 12345678_u64;
    let result =
        new_recruit::update_message_id(&app_state, NEW_GUILD_ID, inserted.id, new_message_id).await;
    assert!(result.is_ok(), "message_id更新に失敗: {:?}", result.err());

    // DBで確認
    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, NEW_GUILD_ID as i64)
        .await
        .unwrap();
    let updated = battle_recruitments::Entity::find_by_id(inserted.id)
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    txn.commit().await.unwrap();
    assert_eq!(updated.message_id, new_message_id as i64);

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), NEW_GUILD_ID as i64, inserted.id).await;
}

/// 2-2: 異常系 - 存在しないrecruitment_id
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_update_message_id_not_found() {
    let app_state = Arc::new(create_test_app_state().await);

    // 存在しないrecruitment_idでmessage_id更新
    let result = new_recruit::update_message_id(&app_state, NEW_GUILD_ID, 99999, 12345678).await;

    assert!(
        result.is_err(),
        "存在しない募集IDでエラーが返りませんでした"
    );
}

// =================================================
// new_recruitment
// =================================================

/// 1-1: 正常系 - ボタン版での新規募集作成
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_button_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 20) as u64;
    let channel_id = (NEW_CHANNEL_ID + 20) as u64;
    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;

    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL",
        None,
        None,
        true,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(result.is_ok(), "ボタン版募集作成に失敗: {:?}", result.err());
    let recruitment_result = result.unwrap();
    assert!(
        !recruitment_result.components.is_empty(),
        "ボタン版なのにcomponentsが空です"
    );
    assert!(
        recruitment_result.reaction_emojis.is_empty(),
        "ボタン版なのにreaction_emojisが設定されています"
    );

    let db_record = battle_recruitments::Entity::find_by_id(recruitment_result.recruitment_id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(db_record.guild_id, guild_id as i64);
    assert_eq!(db_record.channel_id, channel_id as i64);

    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;
}

/// 1-2: 正常系 - リアクション版での新規募集作成
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_reaction_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 21) as u64;
    let channel_id = (NEW_CHANNEL_ID + 21) as u64;
    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;

    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL",
        None,
        None,
        false,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_ok(),
        "リアクション版募集作成に失敗: {:?}",
        result.err()
    );
    let recruitment_result = result.unwrap();
    assert!(
        recruitment_result.components.is_empty(),
        "リアクション版なのにcomponentsがあります"
    );
    assert!(
        !recruitment_result.reaction_emojis.is_empty(),
        "リアクション版なのにreaction_emojisが空です"
    );

    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;
}

/// 1-3: 正常系 - battle_style_id指定あり
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_with_battle_style_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 22) as u64;
    let channel_id = (NEW_CHANNEL_ID + 22) as u64;
    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;

    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL",
        Some(1),
        None,
        true,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_ok(),
        "battle_style指定募集作成に失敗: {:?}",
        result.err()
    );
    let recruitment_result = result.unwrap();
    let db_record = battle_recruitments::Entity::find_by_id(recruitment_result.recruitment_id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(db_record.battle_style_id, 1);

    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;
}

/// 1-4: 正常系 - event_date指定あり
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_with_event_date_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);
    use chrono::{Duration, Utc};

    let guild_id = (NEW_GUILD_ID + 23) as u64;
    let channel_id = (NEW_CHANNEL_ID + 23) as u64;
    let event_date = Utc::now() + Duration::hours(72);
    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;

    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL",
        None,
        Some(event_date),
        true,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_ok(),
        "event_date指定募集作成に失敗: {:?}",
        result.err()
    );
    let recruitment_result = result.unwrap();
    let db_record = battle_recruitments::Entity::find_by_id(recruitment_result.recruitment_id)
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    let diff = (db_record.quest_start_at - event_date).num_minutes().abs();
    assert!(diff <= 1, "quest_start_atが指定日時と一致しません");

    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;
}

/// 1-5: 正常系 - dismissal_times指定あり
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_with_dismissal_times_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 24) as u64;
    let channel_id = (NEW_CHANNEL_ID + 24) as u64;
    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;

    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL",
        None,
        None,
        true,
        Some("30m".to_string()),
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_ok(),
        "dismissal_times指定募集作成に失敗: {:?}",
        result.err()
    );
    let recruitment_result = result.unwrap();
    assert!(
        recruitment_result.message_content.contains("解散"),
        "解散時刻がメッセージに反映されていません"
    );

    cleanup_recruitments_by_guild(app_state.guild_db(), guild_id as i64).await;
}

/// 1-6: 異常系 - 存在しないquest_alias
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_quest_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 10) as u64;
    let channel_id = (NEW_CHANNEL_ID + 10) as u64;

    // 存在しないクエスト名で募集作成を試みる
    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "存在しないクエスト名XXXXXX", // 存在しないクエスト
        None,
        None,
        true,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_err(),
        "存在しないクエストでエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("見つかりませんでした"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}

/// 1-7: 異常系 - 存在しないbattle_style_id
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_new_recruitment_battle_style_not_found() {
    let app_state = Arc::new(create_test_app_state().await);
    let mut mock_gateway = MockTestGateway::new();
    setup_new_recruitment_gateway(&mut mock_gateway);

    let guild_id = (NEW_GUILD_ID + 11) as u64;
    let channel_id = (NEW_CHANNEL_ID + 11) as u64;

    // 存在しないbattle_style_idで募集作成を試みる
    // 注: 実際のクエスト名を使用する必要があるため、DBに存在するクエストを使用
    let result = new_recruit::new_recruitment(
        &app_state,
        &mock_gateway,
        guild_id,
        channel_id,
        "アルバハHL", // 実在するクエスト名（テストDBに登録されていることを想定）
        Some(99999),  // 存在しないbattle_style_id
        None,
        true,
        None,
        TEST_USER_ID,
    )
    .await;

    assert!(
        result.is_err(),
        "存在しないbattle_style_idでエラーが返りませんでした"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("攻略方法ID"),
        "エラーメッセージが期待と異なります: {}",
        err_msg
    );
}
