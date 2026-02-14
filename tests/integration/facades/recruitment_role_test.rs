// 募集通知ロール管理ファサード 結合テスト
//
// 対象: src/facades/recruitment/role_management.rs

use gbf_discord_bot_rs::facades::recruitment::role_management;
use gbf_discord_bot_rs::models::entities::guild_master::{
    all_recruitment_notification_roles, quest_recruitment_notification_roles,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, create_test_app_state};

/// テスト用ID（ロール管理テスト専用）
const ROLE_GUILD_ID: i64 = TEST_GUILD_ID + 1000;
const TEST_ROLE_ID_1: u64 = 111111111;
const TEST_ROLE_ID_2: u64 = 222222222;
const TEST_ROLE_ID_3: u64 = 333333333;

/// テスト用ロールデータを削除
async fn cleanup_role_data(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    let _ = all_recruitment_notification_roles::Entity::delete_many()
        .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;

    let _ = quest_recruitment_notification_roles::Entity::delete_many()
        .filter(quest_recruitment_notification_roles::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

// =================================================
// add_recruitment_notification_roles
// =================================================

/// 11-1: 正常系 - 全募集用ロール追加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_add_all_recruitment_role() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 1) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 全募集用ロールを追加
    let result = role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1, TEST_ROLE_ID_2],
    )
    .await;

    assert!(
        result.is_ok(),
        "全募集用ロール追加に失敗: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 2, "追加数が期待と異なります");

    // DBで確認
    let roles = all_recruitment_notification_roles::Entity::find()
        .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();

    assert_eq!(roles.len(), 2, "登録されたロール数が期待と異なります");
    let role_ids: Vec<i64> = roles.iter().map(|r| r.role_id).collect();
    assert!(role_ids.contains(&(TEST_ROLE_ID_1 as i64)));
    assert!(role_ids.contains(&(TEST_ROLE_ID_2 as i64)));

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

/// 11-2: 正常系 - 特定クエスト用ロール追加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_add_quest_recruitment_role() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 2) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 特定クエスト用ロールを追加
    // 注: テストDBに存在するクエスト名を使用する必要がある
    let result = role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "アルバハHL", // 実在するクエスト名（テストDBに登録されていることを想定）
        vec![TEST_ROLE_ID_1],
    )
    .await;

    assert!(
        result.is_ok(),
        "クエスト用ロール追加に失敗: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "追加数が期待と異なります");

    // DBで確認
    let roles = quest_recruitment_notification_roles::Entity::find()
        .filter(quest_recruitment_notification_roles::Column::GuildId.eq(guild_id as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();

    assert_eq!(roles.len(), 1, "登録されたロール数が期待と異なります");
    assert_eq!(roles[0].role_id, TEST_ROLE_ID_1 as i64);

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

/// 11-3: 正常系 - 重複ロールの追加
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_add_duplicate_role() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 3) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 1回目の追加
    let result1 = role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1],
    )
    .await;

    assert!(
        result1.is_ok(),
        "1回目のロール追加に失敗: {:?}",
        result1.err()
    );
    assert_eq!(result1.unwrap(), 1, "1回目の追加数が期待と異なります");

    // 2回目の追加（重複）
    let result2 = role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1, TEST_ROLE_ID_2],
    )
    .await;

    assert!(
        result2.is_ok(),
        "2回目のロール追加に失敗: {:?}",
        result2.err()
    );
    // TEST_ROLE_ID_1は既に存在するため、TEST_ROLE_ID_2のみが追加される
    assert_eq!(
        result2.unwrap(),
        1,
        "2回目の追加数が期待と異なります（重複は追加されない）"
    );

    // DBで確認（合計2個のロールが登録されているはず）
    let roles = all_recruitment_notification_roles::Entity::find()
        .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();

    assert_eq!(roles.len(), 2, "登録されたロール数が期待と異なります");

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

// =================================================
// remove_recruitment_notification_roles
// =================================================

/// 12-1: 正常系 - ロール削除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_remove_recruitment_role() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 4) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 事前にロールを追加
    role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1, TEST_ROLE_ID_2, TEST_ROLE_ID_3],
    )
    .await
    .unwrap();

    // ロールを削除
    let result = role_management::remove_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1, TEST_ROLE_ID_2],
    )
    .await;

    assert!(result.is_ok(), "ロール削除に失敗: {:?}", result.err());
    assert_eq!(result.unwrap(), 2, "削除数が期待と異なります");

    // DBで確認（TEST_ROLE_ID_3のみ残っているはず）
    let roles = all_recruitment_notification_roles::Entity::find()
        .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id as i64))
        .all(app_state.guild_db())
        .await
        .unwrap();

    assert_eq!(roles.len(), 1, "残っているロール数が期待と異なります");
    assert_eq!(roles[0].role_id, TEST_ROLE_ID_3 as i64);

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

/// 12-2: 正常系 - 未登録ロールの削除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_remove_unregistered_role() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 5) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 未登録のロールを削除（エラーにならないことを確認）
    let result = role_management::remove_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1],
    )
    .await;

    assert!(
        result.is_ok(),
        "未登録ロール削除でエラーが発生: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        0,
        "削除数が期待と異なります（未登録なので0件）"
    );

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

// =================================================
// show_recruitment_notification_roles
// =================================================

/// 13-1: 正常系 - 全ロール設定表示
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_show_roles_with_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 6) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // 全募集用ロールを追加
    role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "すべて",
        vec![TEST_ROLE_ID_1],
    )
    .await
    .unwrap();

    // クエスト用ロールを追加
    role_management::add_recruitment_notification_roles(
        &app_state,
        guild_id,
        "アルバハHL",
        vec![TEST_ROLE_ID_2],
    )
    .await
    .unwrap();

    // ロール設定を取得
    let result = role_management::show_recruitment_notification_roles(&app_state, guild_id).await;

    assert!(result.is_ok(), "ロール設定取得に失敗: {:?}", result.err());

    let settings = result.unwrap();
    assert_eq!(
        settings.all_recruitment_roles.len(),
        1,
        "全募集用ロール数が期待と異なります"
    );
    assert!(
        settings
            .all_recruitment_roles
            .contains(&(TEST_ROLE_ID_1 as i64))
    );
    assert!(
        !settings.quest_recruitment_roles.is_empty(),
        "クエスト用ロールが空です"
    );

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}

/// 13-2: 正常系 - ロール未設定時の表示
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_show_roles_without_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let guild_id = (ROLE_GUILD_ID + 7) as u64;

    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;

    // ロール設定を取得（何も登録されていない状態）
    let result = role_management::show_recruitment_notification_roles(&app_state, guild_id).await;

    assert!(result.is_ok(), "ロール設定取得に失敗: {:?}", result.err());

    let settings = result.unwrap();
    assert_eq!(
        settings.all_recruitment_roles.len(),
        0,
        "全募集用ロール数が期待と異なります（空のはず）"
    );
    assert!(
        settings.quest_recruitment_roles.is_empty(),
        "クエスト用ロールが空ではありません"
    );

    // クリーンアップ
    cleanup_role_data(app_state.guild_db(), guild_id as i64).await;
}
