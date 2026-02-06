// ギルド設定ファサード 結合テスト
//
// 対象: src/facades/guild_settings/guild_settings_facade.rs

use gbf_discord_bot_rs::facades::guild_settings::guild_settings_facade::GuildSettingsFacade;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

use super::test_helper::{TEST_GUILD_ID, create_test_app_state};

/// テスト用ギルドID（ギルド設定テスト専用）
const SETTINGS_TEST_GUILD_ID: i64 = TEST_GUILD_ID + 200;

/// テスト後のクリーンアップ：ギルド設定データを削除
async fn cleanup_guild_settings(db: &sea_orm::DatabaseConnection, guild_id: i64) {
    use gbf_discord_bot_rs::models::entities::guild_master::guild_settings;
    let _ = guild_settings::Entity::delete_many()
        .filter(guild_settings::Column::GuildId.eq(guild_id))
        .exec(db)
        .await;
}

// =================================================
// get_timezones_for_autocomplete（DB不要テスト）
// =================================================

/// 1-1: 正常系 - 部分文字列でタイムゾーン候補取得
#[tokio::test]
#[ignore] // AppState構築にDB接続が必要
async fn test_get_timezones_for_autocomplete_partial_match() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state);

    let options = facade.get_timezones_for_autocomplete("Asia/T");

    assert!(
        !options.is_empty(),
        "Asia/Tにマッチするタイムゾーンが見つかりません"
    );

    // Asia/Tokyoが含まれることを確認
    let has_tokyo = options.iter().any(|o| o.value == "Asia/Tokyo");
    assert!(has_tokyo, "Asia/Tokyoが候補に含まれていません");
}

/// 1-2: 正常系 - 空文字列での候補取得
#[tokio::test]
#[ignore] // AppState構築にDB接続が必要
async fn test_get_timezones_for_autocomplete_empty_string() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state);

    let options = facade.get_timezones_for_autocomplete("");

    // 空文字列でも候補が返ること
    assert!(
        !options.is_empty(),
        "空文字列でタイムゾーン候補が返りません"
    );
    // 最大25件以下であること
    assert!(options.len() <= 25, "候補が25件を超えています");
}

/// 1-3: 正常系 - マッチなしの候補取得
#[tokio::test]
#[ignore] // AppState構築にDB接続が必要
async fn test_get_timezones_for_autocomplete_no_match() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state);

    let options = facade.get_timezones_for_autocomplete("XXXXXXXX");

    assert!(
        options.is_empty(),
        "存在しないタイムゾーンに候補が返りました"
    );
}

// =================================================
// get_timezone（DB必要テスト）
// =================================================

/// 2-2: 正常系 - 未設定時のデフォルト値（Asia/Tokyo）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_timezone_default() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // 未設定のギルドのタイムゾーンを取得
    let result = facade.get_timezone(guild_id).await;
    assert!(result.is_ok(), "タイムゾーン取得に失敗: {:?}", result.err());

    let tz = result.unwrap();
    assert_eq!(
        tz.name(),
        "Asia/Tokyo",
        "デフォルトタイムゾーンがAsia/Tokyoではありません"
    );
}

// =================================================
// get_guild_settings（DB必要テスト）
// =================================================

/// 3-2: 正常系 - 未設定ギルド設定の取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_guild_settings_not_set() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID + 1;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // 未設定のギルド設定を取得
    let result = facade.get_guild_settings(guild_id).await;
    assert!(result.is_ok(), "ギルド設定取得に失敗: {:?}", result.err());

    assert!(
        result.unwrap().is_none(),
        "未設定のギルドに設定が返りました"
    );
}

// =================================================
// set_timezone（DB必要テスト）
// =================================================

/// 4-1: 正常系 - 新規タイムゾーン設定
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_set_timezone_new() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID + 2;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // タイムゾーン設定
    let result = facade
        .set_timezone(guild_id, "America/New_York", "en")
        .await;
    assert!(result.is_ok(), "タイムゾーン設定に失敗: {:?}", result.err());

    let set_result = result.unwrap();
    assert_eq!(set_result.timezone.name(), "America/New_York");

    // DBに反映されたことを確認
    let settings = facade.get_guild_settings(guild_id).await.unwrap();
    assert!(settings.is_some());
    let settings = settings.unwrap();
    assert_eq!(settings.timezone, "America/New_York");
    assert_eq!(settings.locale, "en");

    // クリーンアップ
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;
}

/// 4-2: 正常系 - タイムゾーン変更
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_set_timezone_change() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID + 3;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // 初回設定
    facade
        .set_timezone(guild_id, "Asia/Tokyo", "ja")
        .await
        .unwrap();

    // 変更
    let result = facade.set_timezone(guild_id, "Europe/London", "en").await;
    assert!(result.is_ok());

    // 変更が反映されたことを確認
    let settings = facade.get_guild_settings(guild_id).await.unwrap().unwrap();
    assert_eq!(settings.timezone, "Europe/London");
    assert_eq!(settings.locale, "en");

    // クリーンアップ
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;
}

/// 4-3: 異常系 - 無効なタイムゾーン文字列
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_set_timezone_invalid() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state);
    let guild_id = SETTINGS_TEST_GUILD_ID + 4;

    // 無効なタイムゾーンを設定
    let result = facade
        .set_timezone(guild_id, "Invalid/Timezone", "ja")
        .await;
    assert!(
        result.is_err(),
        "無効なタイムゾーンでエラーが返りませんでした"
    );
}

/// 2-1: 正常系 - 設定済みタイムゾーンの取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_timezone_set_value() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID + 5;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // タイムゾーン設定
    facade
        .set_timezone(guild_id, "America/New_York", "en")
        .await
        .unwrap();

    // 取得して確認
    let result = facade.get_timezone(guild_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "America/New_York");

    // クリーンアップ
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;
}

/// 3-1: 正常系 - 設定済みギルド設定の取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_guild_settings_with_data() {
    let app_state = Arc::new(create_test_app_state().await);
    let facade = GuildSettingsFacade::new(app_state.clone());
    let guild_id = SETTINGS_TEST_GUILD_ID + 6;

    // クリーンアップ（事前）
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;

    // 設定保存
    facade
        .set_timezone(guild_id, "Asia/Tokyo", "ja")
        .await
        .unwrap();

    // 取得して確認
    let result = facade.get_guild_settings(guild_id).await;
    assert!(result.is_ok());
    let settings = result.unwrap().unwrap();
    assert_eq!(settings.timezone, "Asia/Tokyo");
    assert_eq!(settings.locale, "ja");

    // クリーンアップ
    cleanup_guild_settings(app_state.guild_db(), guild_id).await;
}
