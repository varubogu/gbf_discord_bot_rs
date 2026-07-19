/// メッセージキー整合性テスト
///
/// messages.yml のキーをベースとして、yaml_loader.rs との整合性を検証する。
/// DBを使用しないため、#[ignore] は不要。
use std::fs;
use std::path::PathBuf;

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
        .filter(|key| gbf_discord_bot_rs::test_utils::resolve_yaml_message(key, "ja").is_none())
        .cloned()
        .collect();

    assert!(
        unresolved_keys.is_empty(),
        "以下の messages.yml キーが yaml_loader.rs のマッチアームで解決できません。\n\
         メッセージ追加手順に従い、yaml_loader.rs に以下のキーのマッチアームを追加してください:\n{}",
        unresolved_keys.join("\n")
    );
}

/// Task4対象ファイルにユーザー向け直書きフォールバックが残っていないことを検証する
#[test]
fn test_no_hardcoded_user_strings_in_task4_targets() {
    let target_files = [
        "src/events/handlers/component_interaction.rs",
        "src/events/interactions/command_interactions/slash/recruit/recruit_cancel.rs",
        "src/events/interactions/command_interactions/slash/recruit/recruit_change.rs",
        "src/events/interactions/command_interactions/slash/auto_recruit/status.rs",
        "src/presenter/auto_recruitment_presenter.rs",
    ];

    let forbidden_literals = [
        "❌ エラー: 属性を選択してください",
        "❌ エラー: サーバー内でのみ使用できます",
        "募集は既にキャンセルされています。",
        "募集メッセージが削除されています。",
        "指定されたメッセージは募集メッセージではありません。",
        "指定された募集が見つかりません。",
        "開催日時を過ぎているためキャンセルできません。",
        "募集内容を更新しました。",
        "**自動募集参加状況**",
        "**選択中のクエスト:**",
        "**参加可能時間:**",
        "属性を選択してください（複数選択可）",
        "参加したいクエストを選択してください（複数選択可）",
        "時間を選択してください",
        "自動募集の設定が完了しました",
    ];

    for file in target_files {
        let content = fs::read_to_string(file).unwrap_or_else(|e| {
            panic!(
                "テスト対象ファイルの読み込みに失敗しました: {} ({})",
                file, e
            )
        });

        for literal in &forbidden_literals {
            assert!(
                !content.contains(literal),
                "Task4対象ファイルに直書き文言が残っています: file={}, literal={}",
                file,
                literal
            );
        }
    }
}

fn contains_japanese_char(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{3040}'..='\u{30ff}').contains(&c)
            || ('\u{3400}'..='\u{9fff}').contains(&c)
            || ('\u{f900}'..='\u{faff}').contains(&c)
    })
}

/// Task11対象のschedule系/quest_listで、旧フォールバックAPIが残っていないことを検証する
#[test]
fn test_no_hardcoded_fallback_api_in_task11_targets() {
    let target_files = vec![
        PathBuf::from(
            "src/events/interactions/command_interactions/slash/schedule/schedule_generate.rs",
        ),
        PathBuf::from(
            "src/events/interactions/command_interactions/slash/schedule/schedule_global_generate.rs",
        ),
        PathBuf::from(
            "src/events/interactions/command_interactions/slash/schedule/schedule_list.rs",
        ),
        PathBuf::from(
            "src/events/interactions/command_interactions/slash/recruit/recruitment_schedule_list.rs",
        ),
        PathBuf::from("src/events/interactions/command_interactions/slash/quest/quest_list.rs"),
    ];

    for file in target_files {
        let content = fs::read_to_string(&file).unwrap_or_else(|e| {
            panic!(
                "テスト対象ファイルの読み込みに失敗しました: {} ({})",
                file.display(),
                e
            )
        });
        assert!(
            !content.contains("get_message_or_fallback_from_context("),
            "Task11対象ファイルに直書きフォールバックAPIが残っています: {}",
            file.display()
        );
    }
}

/// Task11対象のcategory_setup_facadeで、UI文言を直書きしていないことを検証する
#[test]
fn test_no_hardcoded_category_setup_ui_literals() {
    let target_files = vec![
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/mod.rs"),
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/common.rs"),
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/messages.rs"),
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/register.rs"),
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/unregister.rs"),
        PathBuf::from("src/facades/auto_recruitment/category_setup_facade/change_days.rs"),
    ];

    for file in target_files {
        let content = fs::read_to_string(&file).unwrap_or_else(|e| {
            panic!(
                "テスト対象ファイルの読み込みに失敗しました: {} ({})",
                file.display(),
                e
            )
        });
        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            if (line.contains("MessageContent::text(\"") || line.contains(".with_text(\""))
                && contains_japanese_char(line)
            {
                panic!(
                    "category_setup_facade にUI文言の直書きがあります: {}:{}",
                    file.display(),
                    index + 1
                );
            }

            if line.contains("ButtonContent::new(") {
                let upper = usize::min(index + 6, lines.len());
                let window = lines[index..upper].join("\n");
                if window.contains('\"') && contains_japanese_char(&window) {
                    panic!(
                        "category_setup_facade のButtonContent::new周辺にUI文言の直書きがあります: {}:{}",
                        file.display(),
                        index + 1
                    );
                }
            }
        }
    }
}
