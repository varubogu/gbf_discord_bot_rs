// Integration tests for recruitment v2 functionality (button-based recruitment)

use gbf_discord_bot_rs::services::recruitment::new;
use gbf_discord_bot_rs::types::RecruitmentComponentId;

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
        let custom_id = format!("recruit_join_{}", i);
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

/// ボタン生成のテスト
#[test]
fn test_create_recruitment_buttons_six_elements() {
    let buttons = new::create_recruitment_buttons("6属性");

    // 3行のボタンが生成されることを確認
    assert_eq!(buttons.len(), 3);

    // 各行のボタン数を確認
    // 行1: 属性1-3 (3個)
    // 行2: 属性4-6 (3個)
    // 行3: 全属性可能 + 全て取り消し (2個)
    match &buttons[0] {
        poise::serenity_prelude::CreateActionRow::Buttons(btns) => {
            assert_eq!(btns.len(), 3, "第1行は3つのボタンを持つべき");
        }
        _ => panic!("第1行はボタン行であるべき"),
    }

    match &buttons[1] {
        poise::serenity_prelude::CreateActionRow::Buttons(btns) => {
            assert_eq!(btns.len(), 3, "第2行は3つのボタンを持つべき");
        }
        _ => panic!("第2行はボタン行であるべき"),
    }

    match &buttons[2] {
        poise::serenity_prelude::CreateActionRow::Buttons(btns) => {
            assert_eq!(btns.len(), 2, "第3行は2つのボタンを持つべき");
        }
        _ => panic!("第3行はボタン行であるべき"),
    }
}

#[test]
fn test_create_recruitment_buttons_simple() {
    let buttons = new::create_recruitment_buttons("シンプル");

    // 1行のボタンが生成されることを確認
    assert_eq!(buttons.len(), 1);

    // 参加ボタン + 全て取り消しボタン (2個)
    match &buttons[0] {
        poise::serenity_prelude::CreateActionRow::Buttons(btns) => {
            assert_eq!(btns.len(), 2, "シンプル参加は2つのボタンを持つべき");
        }
        _ => panic!("ボタン行であるべき"),
    }
}

#[test]
fn test_create_recruitment_buttons_other_battle_style() {
    // 6属性以外の攻略方法はシンプル参加と同じ
    let buttons = new::create_recruitment_buttons("ワンパン");

    assert_eq!(buttons.len(), 1);

    match &buttons[0] {
        poise::serenity_prelude::CreateActionRow::Buttons(btns) => {
            assert_eq!(btns.len(), 2, "ワンパンは2つのボタンを持つべき");
        }
        _ => panic!("ボタン行であるべき"),
    }
}

/// メッセージ内容作成のテスト
#[test]
fn test_create_message_content() {
    use chrono::Utc;

    let quest_name = "ルシファーHL";
    let battle_style_name = "6属性";
    let expiry_date = Utc::now() + chrono::Duration::hours(3);
    let timezone = chrono_tz::Asia::Tokyo;

    let message = new::create_message_content(quest_name, battle_style_name, &expiry_date, timezone);

    // クエスト名が含まれていることを確認
    assert!(message.contains(quest_name));

    // 6属性の場合、属性選択メッセージが含まれることを確認
    assert!(message.contains("参加属性を選んでください"));
}

#[test]
fn test_create_message_content_simple() {
    use chrono::Utc;

    let quest_name = "ルシファーHL";
    let battle_style_name = "シンプル";
    let expiry_date = Utc::now() + chrono::Duration::hours(3);
    let timezone = chrono_tz::Asia::Tokyo;

    let message = new::create_message_content(quest_name, battle_style_name, &expiry_date, timezone);

    // クエスト名が含まれていることを確認
    assert!(message.contains(quest_name));

    // シンプルの場合、属性選択メッセージが含まれないことを確認
    assert!(!message.contains("参加属性を選んでください"));
}

// Note: Service層とRepository層のテストは、実際のDB接続またはモックが必要なため、
// ここでは単純なロジックのテストのみを実装しています。
// 実際のDB操作を含むテストは、テスト用DBを使用した統合テストとして別途実装する必要があります。
