/// メッセージキー整合性テスト
///
/// messages.yml のキーをベースとして、yaml_loader.rs との整合性を検証する。
/// DBを使用しないため、#[ignore] は不要。
use std::fs;

/// messages.yml の全キーが yaml_loader.rs のマッチアームで解決されることを検証する
///
/// このテストは以下の整合性をチェックする:
/// - messages.yml に定義されたキー → `yaml_loader::get_yaml_message()` で解決可能か
///
/// これは「YAMLにキーを追加したが yaml_loader.rs のマッチアームを追加し忘れた」
/// ケースを検出するためのテストである。
///
/// 既存の `test_all_message_ids_exist_in_yaml`（message_text_id.rs 内）が
/// 「MessageTextId 列挙型 → YAML」方向をチェックするのに対し、
/// このテストは逆方向「YAML → yaml_loader.rs」をチェックする。
#[test]
fn test_all_yaml_keys_resolved_by_yaml_loader() {
    let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/locales/messages.yml");
    let yaml_content =
        fs::read_to_string(yaml_path).expect("locales/messages.yml が見つかりません");

    // messages.yml から全トップレベルキーを抽出
    // トップレベルキーの条件:
    // - インデントなし（行頭がスペース・タブでない）
    // - ':' で終わる
    // - "_version" など内部メタデータキー（'_' 始まり）は除外
    let yaml_keys: Vec<String> = yaml_content
        .lines()
        .filter_map(|line| {
            if line.starts_with(' ') || line.starts_with('\t') {
                return None;
            }
            let trimmed = line.trim();
            let key = trimmed.strip_suffix(':')?;
            if key.starts_with('_') {
                return None;
            }
            Some(key.to_string())
        })
        .collect();

    assert!(
        !yaml_keys.is_empty(),
        "messages.yml からキーを抽出できませんでした"
    );

    // 各キーが yaml_loader で解決されるか確認し、未解決のキーを収集
    let unresolved_keys: Vec<String> = yaml_keys
        .iter()
        .filter(|key| {
            gbf_discord_bot_rs::test_utils::resolve_yaml_message(key, "ja").is_none()
        })
        .cloned()
        .collect();

    assert!(
        unresolved_keys.is_empty(),
        "以下の messages.yml キーが yaml_loader.rs のマッチアームで解決できません。\n\
         メッセージ追加手順に従い、yaml_loader.rs に以下のキーのマッチアームを追加してください:\n{}",
        unresolved_keys.join("\n")
    );
}
