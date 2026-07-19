// ギルド管理ファサード 結合テスト
//
// 対象: src/facades/guild/guild_management_facade.rs

use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::infrastructure::database::session::set_current_guild_id;
use gbf_discord_bot_rs::models::entities::guild_master::guilds;
use gbf_discord_bot_rs::types::AppState;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, create_test_app_state, get_test_guild_role_db};

/// テスト用ギルドID（ギルド管理テスト専用）
const GUILD_TEST_ID: i64 = TEST_GUILD_ID + 100;

/// テスト後のクリーンアップ：ギルドデータを削除
async fn cleanup_guild(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    let _ = guilds::Entity::delete_many()
        .filter(guilds::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

/// 1-1: 正常系 - 新規ギルド登録
#[tokio::test]
async fn test_register_new_guild_success() {
    let app_state: Arc<AppState> = Arc::new(create_test_app_state().await);
    let facade = GuildManagementFacade::new(app_state.clone());
    let guild_id = GUILD_TEST_ID;

    // クリーンアップ（事前）
    cleanup_guild(app_state.guild_db(), guild_id).await;

    // 新規ギルド登録
    let result = facade.register_new_guild(guild_id, "テストギルド").await;
    assert!(result.is_ok(), "新規ギルド登録に失敗: {:?}", result.err());

    // DBに登録されたことを確認
    let found = guilds::Entity::find()
        .filter(guilds::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap();
    assert!(found.is_some(), "ギルドがDBに登録されていません");

    // クリーンアップ
    cleanup_guild(app_state.guild_db(), guild_id).await;
}

/// Guildロールは設定済みギルドのデータだけを参照できる
#[tokio::test]
async fn test_guild_rls_isolates_other_guild_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildManagementFacade::new(app_state.clone());
    let guild_id = GUILD_TEST_ID + 90;
    cleanup_guild(app_state.guild_db(), guild_id).await;
    facade
        .register_new_guild(guild_id, "RLS検証用ギルド")
        .await
        .unwrap();

    let guild_db = get_test_guild_role_db().await;
    let own_txn = guild_db.begin().await.unwrap();
    set_current_guild_id(&own_txn, guild_id).await.unwrap();
    assert!(
        guilds::Entity::find_by_id(guild_id)
            .one(&own_txn)
            .await
            .unwrap()
            .is_some()
    );
    own_txn.rollback().await.unwrap();

    let other_txn = guild_db.begin().await.unwrap();
    set_current_guild_id(&other_txn, guild_id + 1)
        .await
        .unwrap();
    assert!(
        guilds::Entity::find_by_id(guild_id)
            .one(&other_txn)
            .await
            .unwrap()
            .is_none()
    );
    other_txn.rollback().await.unwrap();

    cleanup_guild(app_state.guild_db(), guild_id).await;
}

/// 1-2: 正常系 - 既存ギルドの再登録（冪等性）
#[tokio::test]
async fn test_register_existing_guild_idempotent() {
    let app_state: Arc<AppState> = Arc::new(create_test_app_state().await);
    let facade = GuildManagementFacade::new(app_state.clone());
    let guild_id = GUILD_TEST_ID + 1;

    // クリーンアップ（事前）
    cleanup_guild(app_state.guild_db(), guild_id).await;

    // 1回目の登録
    let result1 = facade.register_new_guild(guild_id, "テストギルド1").await;
    assert!(result1.is_ok());

    // 2回目の登録（同一guild_id、異なるギルド名）
    let result2 = facade.register_new_guild(guild_id, "テストギルド2").await;
    assert!(
        result2.is_ok(),
        "既存ギルドの再登録でエラーが発生: {:?}",
        result2.err()
    );

    // クリーンアップ
    cleanup_guild(app_state.guild_db(), guild_id).await;
}

/// 1-3: 正常系 - 異なるギルド名での更新
#[tokio::test]
async fn test_register_guild_updates_name() {
    let app_state: Arc<AppState> = Arc::new(create_test_app_state().await);
    let facade = GuildManagementFacade::new(app_state.clone());
    let guild_id = GUILD_TEST_ID + 2;

    // クリーンアップ（事前）
    cleanup_guild(app_state.guild_db(), guild_id).await;

    // 初回登録
    facade
        .register_new_guild(guild_id, "旧ギルド名")
        .await
        .unwrap();

    // ギルド名変更して再登録
    facade
        .register_new_guild(guild_id, "新ギルド名")
        .await
        .unwrap();

    // DBのギルド名が更新されたことを確認
    let found = guilds::Entity::find()
        .filter(guilds::Column::GuildId.eq(guild_id))
        .one(app_state.guild_db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.name, "新ギルド名");

    // クリーンアップ
    cleanup_guild(app_state.guild_db(), guild_id).await;
}
