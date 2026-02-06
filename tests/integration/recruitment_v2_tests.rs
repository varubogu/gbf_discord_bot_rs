// Integration tests for recruitment v2 functionality (button-based recruitment)

use gbf_discord_bot_rs::services::guild_environment_service::ElementEmojis;
use gbf_discord_bot_rs::services::recruitment::new;
use gbf_discord_bot_rs::types::RecruitmentComponentId;
use gbf_discord_bot_rs::types::discord::ComponentContent;

/// ComponentIdのパーステスト
#[test]
fn test_component_id_parse_join() {
    let result = RecruitmentComponentId::parse("recruit_join");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RecruitmentComponentId::Join);
}

#[test]
fn test_component_id_parse_join_element() {
    for i in 1..=6 {
        let custom_id = format!("recruit_join_{i}");
        let result = RecruitmentComponentId::parse(&custom_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RecruitmentComponentId::JoinElement(i));
    }
}

#[test]
fn test_component_id_parse_join_all_elements() {
    let result = RecruitmentComponentId::parse("recruit_join_0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RecruitmentComponentId::JoinAllElements);
}

#[test]
fn test_component_id_parse_leave_all() {
    let result = RecruitmentComponentId::parse("recruit_leave_all");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RecruitmentComponentId::LeaveAll);
}

#[test]
fn test_component_id_parse_invalid() {
    // 不正なプレフィックス
    let result = RecruitmentComponentId::parse("invalid_prefix");
    assert!(result.is_err());

    // 不正な要素ID
    let result = RecruitmentComponentId::parse("recruit_join_7");
    assert!(result.is_err());

    let result = RecruitmentComponentId::parse("recruit_join_-1");
    assert!(result.is_err());

    // 不正なフォーマット
    let result = RecruitmentComponentId::parse("recruit_join_abc");
    assert!(result.is_err());
}

/// ボタン生成のテスト（ドメイン型版）
#[test]
fn test_create_recruitment_buttons_six_elements() {
    let element_emojis = ElementEmojis::default_emojis();
    let buttons = new::create_recruitment_buttons("6属性", &element_emojis);

    // 3行のボタンが生成されることを確認
    assert_eq!(buttons.len(), 3);

    // 各行のボタン数を確認（ドメイン型ActionRowContentを使用）
    // 行1: 属性1-3 (3個)
    // 行2: 属性4-6 (3個)
    // 行3: 全属性可能 + 全て取り消し (2個)
    let button_count_row1 = buttons[0]
        .components
        .iter()
        .filter(|c| matches!(c, ComponentContent::Button(_)))
        .count();
    assert_eq!(button_count_row1, 3, "第1行は3つのボタンを持つべき");

    let button_count_row2 = buttons[1]
        .components
        .iter()
        .filter(|c| matches!(c, ComponentContent::Button(_)))
        .count();
    assert_eq!(button_count_row2, 3, "第2行は3つのボタンを持つべき");

    let button_count_row3 = buttons[2]
        .components
        .iter()
        .filter(|c| matches!(c, ComponentContent::Button(_)))
        .count();
    assert_eq!(button_count_row3, 2, "第3行は2つのボタンを持つべき");
}

#[test]
fn test_create_recruitment_buttons_simple() {
    let element_emojis = ElementEmojis::default_emojis();
    let buttons = new::create_recruitment_buttons("シンプル", &element_emojis);

    // 1行のボタンが生成されることを確認
    assert_eq!(buttons.len(), 1);

    // 参加ボタン + 全て取り消しボタン (2個)（ドメイン型を使用）
    let button_count = buttons[0]
        .components
        .iter()
        .filter(|c| matches!(c, ComponentContent::Button(_)))
        .count();
    assert_eq!(button_count, 2, "シンプル参加は2つのボタンを持つべき");
}

#[test]
fn test_create_recruitment_buttons_other_battle_style() {
    // 6属性以外の攻略方法はシンプル参加と同じ
    let element_emojis = ElementEmojis::default_emojis();
    let buttons = new::create_recruitment_buttons("ワンパン", &element_emojis);

    assert_eq!(buttons.len(), 1);

    let button_count = buttons[0]
        .components
        .iter()
        .filter(|c| matches!(c, ComponentContent::Button(_)))
        .count();
    assert_eq!(button_count, 2, "ワンパンは2つのボタンを持つべき");
}

// Note: create_message_content関数のテストは、実際のDB接続が必要なため、
// 統合テストとしてテスト用DBを使用する環境でのみ実行可能です。
// 現在のテスト環境ではDB接続が利用できないため、これらのテストはスキップされています。
//
// 将来的には以下のいずれかの方法でテストを実装すべきです：
// 1. テスト用DBを準備し、統合テストとして実行
// 2. MessageServiceをモック化して単体テストとして実行
// 3. テストコンテナを使用してDB環境を自動セットアップ

// Note: Service層とRepository層のテストは、実際のDB接続またはモックが必要なため、
// ここでは単純なロジックのテストのみを実装しています。
// 実際のDB操作を含むテストは、テスト用DBを使用した統合テストとして別途実装する必要があります。
