// チャンネル管理ファサード 結合テスト
//
// 対象: src/facades/channel/channel_management_facade.rs

use gbf_discord_bot_rs::facades::channel::channel_management_facade::ChannelManagementFacade;
use gbf_discord_bot_rs::facades::guild::guild_management_facade::GuildManagementFacade;
use gbf_discord_bot_rs::types::AppState;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

use super::test_helper::{TEST_CHANNEL_ID, TEST_GUILD_ID, create_test_app_state};

/// テスト用ギルドID（チャンネルテスト専用）
const CH_TEST_GUILD_ID: i64 = TEST_GUILD_ID + 300;
/// テスト用チャンネルID
const CH_TEST_CHANNEL_ID: i64 = TEST_CHANNEL_ID + 300;

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

/// テスト用ギルドを事前登録
async fn setup_guild(app_state: &Arc<AppState>, guild_id: i64) {
    let guild_facade = GuildManagementFacade::new(app_state.clone());
    guild_facade
        .register_new_guild(guild_id, "テスト用ギルド")
        .await
        .unwrap();
}

/// 全テストデータをクリーンアップ
async fn cleanup_all(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    cleanup_guild_channels(db, guild_id).await;
    cleanup_guild(db, guild_id).await;
}

// =================================================
// get_channel_types_for_autocomplete
// =================================================

/// 1-1: 正常系 - チャンネル種別一覧を取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_channel_types_for_autocomplete() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state);

    let result = facade.get_channel_types_for_autocomplete().await;
    assert!(
        result.is_ok(),
        "チャンネル種別取得に失敗: {:?}",
        result.err()
    );

    // マスターデータにチャンネル種別が存在する場合、結果が返ること
    let options = result.unwrap();
    // チャンネル種別は事前にマスターデータとして登録されている前提
    // テスト環境の状況に応じてアサーションを調整
    println!("取得したチャンネル種別数: {}", options.len());
}

/// 1-2: 正常系 - 空のテーブルからの取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_channel_types_for_autocomplete_empty() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());

    // channel_typesテーブルが空の場合を想定
    // （実際のテスト環境ではマスターデータがあるため、空にするのは困難）
    // 結果がエラーにならずVecが返ることを確認
    let result = facade.get_channel_types_for_autocomplete().await;
    assert!(
        result.is_ok(),
        "空のテーブルでもエラーにならないこと: {:?}",
        result.err()
    );

    // 空のVecまたはマスターデータが返る
    let options = result.unwrap();
    println!("取得したチャンネル種別数: {}", options.len());
}

// =================================================
// register_channel
// =================================================

/// 2-1: 正常系 - 新規ギルド・新規チャンネル登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_channel_new_guild() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID;
    let channel_id = CH_TEST_CHANNEL_ID;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // チャンネル登録
    let result = facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await;
    assert!(result.is_ok(), "チャンネル登録に失敗: {:?}", result.err());

    let registration = result.unwrap();
    assert_eq!(registration.channel_id, channel_id);
    assert!(!registration.channel_type_name.is_empty());

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 2-2: 正常系 - 既存ギルドへのチャンネル登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_channel_existing_guild() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 10;
    let channel_id = CH_TEST_CHANNEL_ID + 10;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // ギルドを事前登録
    setup_guild(&app_state, guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        cleanup_all(app_state.guild_db(), guild_id).await;
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // チャンネル登録（ギルドは既存）
    let result = facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await;
    assert!(
        result.is_ok(),
        "既存ギルドへのチャンネル登録に失敗: {:?}",
        result.err()
    );

    let registration = result.unwrap();
    assert_eq!(registration.channel_id, channel_id);

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 2-3: 正常系 - 既存チャンネルの上書き登録
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_channel_overwrite() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 1;
    let channel_id_1 = CH_TEST_CHANNEL_ID + 1;
    let channel_id_2 = CH_TEST_CHANNEL_ID + 2;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // 1回目の登録
    facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id_1,
        )
        .await
        .unwrap();

    // 同じ種別で異なるチャンネルIDを登録（上書き）
    let result = facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id_2,
        )
        .await;
    assert!(
        result.is_ok(),
        "チャンネル上書き登録に失敗: {:?}",
        result.err()
    );

    let registration = result.unwrap();
    assert_eq!(registration.channel_id, channel_id_2);

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 2-4: 異常系 - 存在しないchannel_type_id
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_channel_invalid_type() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 2;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 存在しないchannel_type_idで登録
    let result = facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            99999,
            CH_TEST_CHANNEL_ID,
        )
        .await;
    assert!(
        result.is_err(),
        "存在しないchannel_type_idでエラーが返りませんでした"
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 2-5: 正常系 - 登録結果にsettings_displayが含まれる
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_register_channel_settings_display() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 11;
    let channel_id = CH_TEST_CHANNEL_ID + 11;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // チャンネル登録
    let result = facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await;
    assert!(result.is_ok(), "チャンネル登録に失敗: {:?}", result.err());

    let registration = result.unwrap();
    // settings_displayに登録済みチャンネル情報が反映されていることを確認
    assert!(
        !registration.settings_display.settings.is_empty(),
        "settings_displayが空です"
    );
    // 登録したチャンネルの情報が含まれていることを確認
    let has_channel = registration
        .settings_display
        .settings
        .iter()
        .any(|s| s.channel_id == Some(channel_id));
    assert!(
        has_channel,
        "settings_displayに登録したchannel_idが含まれていません"
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

// =================================================
// unregister_channel
// =================================================

/// 3-1: 正常系 - チャンネル登録解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_channel_success() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 3;
    let channel_id = CH_TEST_CHANNEL_ID + 3;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // 事前登録
    facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await
        .unwrap();

    // 登録解除
    let result = facade.unregister_channel(guild_id, channel_type_id).await;
    assert!(
        result.is_ok(),
        "チャンネル登録解除に失敗: {:?}",
        result.err()
    );

    let unregistration = result.unwrap();
    assert_eq!(unregistration.old_channel_id, channel_id);

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 3-2: 異常系 - 未登録チャンネルの解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_channel_not_registered() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 4;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // ギルドを登録（チャンネルは未登録）
    setup_guild(&app_state, guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        cleanup_all(app_state.guild_db(), guild_id).await;
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // 未登録チャンネルの解除
    let result = facade.unregister_channel(guild_id, channel_type_id).await;
    assert!(
        result.is_err(),
        "未登録チャンネルの解除でエラーが返りませんでした"
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 3-3: 異常系 - 存在しないchannel_type_idでの解除
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_channel_invalid_type() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 5;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // ギルドを登録
    setup_guild(&app_state, guild_id).await;

    // 存在しないchannel_type_idで解除
    let result = facade.unregister_channel(guild_id, 99999).await;
    assert!(
        result.is_err(),
        "存在しないchannel_type_idでエラーが返りませんでした"
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 3-4: 正常系 - 解除結果にold_channel_idが含まれる
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_unregister_channel_old_channel_id() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 12;
    let channel_id = CH_TEST_CHANNEL_ID + 12;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // 事前登録
    facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await
        .unwrap();

    // 登録解除
    let result = facade.unregister_channel(guild_id, channel_type_id).await;
    assert!(
        result.is_ok(),
        "チャンネル登録解除に失敗: {:?}",
        result.err()
    );

    let unregistration = result.unwrap();
    // old_channel_idが解除前のchannel_idと一致することを確認
    assert_eq!(
        unregistration.old_channel_id, channel_id,
        "old_channel_idが解除前のchannel_idと一致しません"
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

// =================================================
// show_channel_settings
// =================================================

/// 4-1: 正常系 - チャンネル設定表示（登録あり）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_show_channel_settings_with_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 6;
    let channel_id = CH_TEST_CHANNEL_ID + 6;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 有効なchannel_type_idを取得
    let types = facade.get_channel_types_for_autocomplete().await.unwrap();
    if types.is_empty() {
        println!("テストスキップ: チャンネル種別マスターデータがありません");
        return;
    }
    let channel_type_id: i32 = types[0].value.parse().unwrap();

    // チャンネル登録
    facade
        .register_channel(
            guild_id,
            "テストギルド".to_string(),
            channel_type_id,
            channel_id,
        )
        .await
        .unwrap();

    // 設定表示
    let result = facade.show_channel_settings(guild_id).await;
    assert!(
        result.is_ok(),
        "チャンネル設定表示に失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}

/// 4-2: 正常系 - チャンネル設定表示（登録なし）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_show_channel_settings_empty() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = ChannelManagementFacade::new(app_state.clone());
    let guild_id = CH_TEST_GUILD_ID + 7;

    // クリーンアップ（事前）
    cleanup_all(app_state.guild_db(), guild_id).await;

    // 設定表示（データなし）
    let result = facade.show_channel_settings(guild_id).await;
    assert!(
        result.is_ok(),
        "チャンネル設定表示に失敗: {:?}",
        result.err()
    );

    // クリーンアップ
    cleanup_all(app_state.guild_db(), guild_id).await;
}
