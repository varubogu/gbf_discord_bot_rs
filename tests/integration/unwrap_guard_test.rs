use std::fs;
use std::path::Path;

/// Task13対象ファイルに `unwrap(` が再混入していないことを確認する
#[test]
fn test_no_unwrap_in_task13_targets() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let targets = [
        "src/events/interactions/command_interactions/slash/recruit/recruitment_schedule_create.rs",
        "src/services/recruitment/schedule/schedule_create_service.rs",
        "src/services/schedule/timezone_converter.rs",
        "src/services/schedule/recruitment_schedule_service.rs",
        "src/services/schedule/schedule_calculator.rs",
        "src/presenter/schedule_presenter.rs",
        "src/services/message/message_service.rs",
        "src/services/unified_datetime_parser.rs",
        "src/services/schedule/dismissal_task_executor.rs",
        "src/lib.rs",
    ];

    let mut violations = Vec::new();

    for target in targets {
        let file_path = repo_root.join(target);
        let content =
            fs::read_to_string(&file_path).expect("Task13対象ファイルの読み込みに失敗しました");

        let test_module_start = content
            .lines()
            .position(|line| line.trim_start().starts_with("#[cfg(test)]"))
            .map(|idx| idx + 1)
            .unwrap_or(usize::MAX);

        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1;
            if line_no >= test_module_start {
                break;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with('*') {
                continue;
            }

            if trimmed.contains("unwrap(") {
                violations.push(format!("{target}:{line_no}: {trimmed}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Task13対象ファイルに unwrap が残存しています:\n{}",
        violations.join("\n")
    );
}
