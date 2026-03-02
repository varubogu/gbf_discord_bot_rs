// 募集クエリファサード 結合テスト
//
// 対象:
// - src/facades/recruitment/quest_list.rs
// - src/facades/recruitment/battle_style_list.rs

use gbf_discord_bot_rs::facades::recruitment::battle_style_list;
use gbf_discord_bot_rs::facades::recruitment::quest_list;
use std::sync::Arc;

use super::test_helper::create_test_app_state;

// =================================================
// search_quests_for_autocomplete
// =================================================

/// 9-1: 正常系 - クエスト検索（マスターデータがある場合）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_search_quests_for_autocomplete() {
    let app_state = Arc::new(create_test_app_state().await);

    // 空文字で全クエスト検索
    let options = quest_list::search_quests_for_autocomplete(&app_state, 0, "").await;

    // マスターデータの状態に依存するが、エラーにならないことを確認
    println!("取得したクエスト数: {}", options.len());
}

/// 9-2: 正常系 - マッチなし
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_search_quests_no_match() {
    let app_state = Arc::new(create_test_app_state().await);

    let options =
        quest_list::search_quests_for_autocomplete(&app_state, 0, "存在しないクエスト名XXXXX")
            .await;

    assert!(options.is_empty(), "存在しないクエスト名に結果が返りました");
}

// =================================================
// list_quests_for_select_with_db
// =================================================

/// クエスト一覧取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_list_quests_for_select() {
    let app_state = Arc::new(create_test_app_state().await);

    let quests = quest_list::list_quests_for_select(&app_state).await;

    // 25件以下であること
    assert!(quests.len() <= 25, "クエスト一覧が25件を超えています");
    println!("取得したクエスト数: {}", quests.len());
}

// =================================================
// get_battle_styles_for_autocomplete
// =================================================

/// 10-1: 正常系 - 攻略方法一覧取得
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_battle_styles_for_autocomplete() {
    let app_state = Arc::new(create_test_app_state().await);

    let options = battle_style_list::get_battle_styles_for_autocomplete(&app_state).await;

    // マスターデータの状態に依存するが、エラーにならないことを確認
    println!("取得した攻略方法数: {}", options.len());
}

// =================================================
// list_battle_styles_for_select_with_db
// =================================================

/// 攻略方法一覧取得（セレクト用）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_list_battle_styles_for_select() {
    let app_state = Arc::new(create_test_app_state().await);

    let styles = battle_style_list::list_battle_styles_for_select(&app_state).await;

    println!("取得した攻略方法数: {}", styles.len());
}

/// 攻略方法名をID指定で取得（存在しないID）
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_battle_style_name_by_invalid_id() {
    let app_state = Arc::new(create_test_app_state().await);

    let name = battle_style_list::get_battle_style_name_by_id(&app_state, 99999).await;
    assert!(name.is_none(), "存在しないIDで攻略方法名が返りました");
}
