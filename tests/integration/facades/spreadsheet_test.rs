// スプレッドシートファサード 結合テスト

use gbf_discord_bot_rs::facades::spreadsheet::GuildSpreadsheetRegistrationFacade;
use gbf_discord_bot_rs::infrastructure::database::repositories::SeaOrmGuildSpreadsheetConfigRepository;
use sea_orm::EntityTrait;

use super::test_helper::{TEST_GUILD_ID, get_test_admin_db, get_test_guild_db};

/// 1-1: 異常系：必須環境変数なし
///
/// GOOGLE_SERVICE_ACCOUNT_KEY_FILE未設定の場合、FacadeError::Initializationが返る
#[tokio::test]
async fn test_new_missing_env_var() {
    // 環境変数を削除
    unsafe {
        std::env::remove_var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE");
    }

    let db = get_test_guild_db().await;
    let result = GuildSpreadsheetRegistrationFacade::new(db);

    // 初期化エラーが返る
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            gbf_discord_bot_rs::errors::FacadeError::Initialization { message } => {
                assert!(message.contains("GOOGLE_SERVICE_ACCOUNT_KEY_FILE"));
            }
            _ => panic!("Expected FacadeError::Initialization"),
        }
    }
}

/// 2-1: 異常系：URL形式不正
///
/// 不正なスプレッドシートURLを指定した場合、URL抽出時点でエラーが返り、
/// DBに設定レコードが作成されない
#[tokio::test]
async fn test_register_invalid_url() {
    // テスト用環境変数を設定（ダミーファイルパス）
    unsafe {
        std::env::set_var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE", "/tmp/dummy.json");
    }

    let guild_db = get_test_guild_db().await;
    let admin_db = get_test_admin_db().await;

    // テスト前のクリーンアップ
    use gbf_discord_bot_rs::models::entities::guild_master::{
        guild_spreadsheet_exports, guild_spreadsheet_imports,
    };
    let _ = guild_spreadsheet_imports::Entity::delete_many()
        .exec(&admin_db)
        .await;
    let _ = guild_spreadsheet_exports::Entity::delete_many()
        .exec(&admin_db)
        .await;

    let facade = GuildSpreadsheetRegistrationFacade::new(guild_db.clone())
        .expect("Facadeの初期化に失敗しました");

    // 不正なURL
    let invalid_url = "https://invalid-url.com/";
    let valid_url = "https://docs.google.com/spreadsheets/d/ABC123/edit";

    let result = facade
        .register_guild_spreadsheets(TEST_GUILD_ID, invalid_url, valid_url)
        .await;

    // エラーが返る
    assert!(result.is_err());

    // DBに設定レコードが作成されていない
    use gbf_discord_bot_rs::repository::GuildSpreadsheetConfigRepositoryTrait;
    let repo = SeaOrmGuildSpreadsheetConfigRepository::new();
    let config = repo
        .find_import_spreadsheet_id(&admin_db, TEST_GUILD_ID)
        .await
        .ok();
    assert!(
        config.is_none() || config == Some(None),
        "DBにレコードが作成されてはいけない"
    );
}

/// 2-2: 異常系：片方のみ不正URL
///
/// load/pushの一方のみ不正な場合、エラーが返り、
/// DBに設定レコードが作成されない（部分登録なし）
#[tokio::test]
async fn test_register_one_invalid_url() {
    // テスト用環境変数を設定（ダミーファイルパス）
    unsafe {
        std::env::set_var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE", "/tmp/dummy.json");
    }

    let guild_db = get_test_guild_db().await;
    let admin_db = get_test_admin_db().await;

    // テスト前のクリーンアップ
    use gbf_discord_bot_rs::models::entities::guild_master::{
        guild_spreadsheet_exports, guild_spreadsheet_imports,
    };
    let _ = guild_spreadsheet_imports::Entity::delete_many()
        .exec(&admin_db)
        .await;
    let _ = guild_spreadsheet_exports::Entity::delete_many()
        .exec(&admin_db)
        .await;

    let facade = GuildSpreadsheetRegistrationFacade::new(guild_db.clone())
        .expect("Facadeの初期化に失敗しました");

    // load_urlは正常、push_urlが不正
    let valid_url = "https://docs.google.com/spreadsheets/d/ABC123/edit";
    let invalid_url = "not-a-url";

    let result = facade
        .register_guild_spreadsheets(TEST_GUILD_ID, valid_url, invalid_url)
        .await;

    // エラーが返る
    assert!(result.is_err());

    // DBに設定レコードが作成されていない
    use gbf_discord_bot_rs::repository::GuildSpreadsheetConfigRepositoryTrait;
    let repo = SeaOrmGuildSpreadsheetConfigRepository::new();
    let config = repo
        .find_import_spreadsheet_id(&admin_db, TEST_GUILD_ID)
        .await
        .ok();
    assert!(
        config.is_none() || config == Some(None),
        "DBにレコードが作成されてはいけない"
    );
}
