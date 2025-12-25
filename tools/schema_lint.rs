#!/usr/bin/env rust-script
//! スキーマ整合性検証ツール
//!
//! エンティティ定義のschema_name属性と、生成されたget_schema_name関数の整合性を検証します。
//!
//! 使用方法:
//!   cargo run --bin schema_lint

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

/// エンティティファイルからスキーマ情報を抽出
fn extract_schema_info_from_entities() -> HashMap<String, String> {
    let mut schema_map = HashMap::new();
    let entities_dir = Path::new("src/models/entities");

    if !entities_dir.exists() {
        eprintln!("エラー: エンティティディレクトリが見つかりません: {:?}", entities_dir);
        return schema_map;
    }

    let entries = match fs::read_dir(entities_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("エンティティディレクトリの読み取りに失敗: {}", e);
            return schema_map;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("ファイル読み取りエラー {:?}: {}", path, e);
                continue;
            }
        };

        if let Some((schema_name, table_name)) = parse_entity_file(&content) {
            schema_map.insert(table_name, schema_name);
        }
    }

    schema_map
}

/// エンティティファイルの内容からschema_nameとtable_nameを抽出
fn parse_entity_file(content: &str) -> Option<(String, String)> {
    let mut schema_name = None;
    let mut table_name = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#[sea_orm(") {
            if let Some(start) = trimmed.find("schema_name = \"") {
                let start_idx = start + "schema_name = \"".len();
                if let Some(end_idx) = trimmed[start_idx..].find('"') {
                    schema_name = Some(trimmed[start_idx..start_idx + end_idx].to_string());
                }
            }

            if let Some(start) = trimmed.find("table_name = \"") {
                let start_idx = start + "table_name = \"".len();
                if let Some(end_idx) = trimmed[start_idx..].find('"') {
                    table_name = Some(trimmed[start_idx..start_idx + end_idx].to_string());
                }
            }
        }
    }

    match (schema_name, table_name) {
        (Some(s), Some(t)) => Some((s, t)),
        _ => None,
    }
}

fn main() {
    println!("=== スキーマ整合性検証ツール ===\n");

    // エンティティからスキーマ情報を抽出
    let entity_schemas = extract_schema_info_from_entities();

    if entity_schemas.is_empty() {
        eprintln!("エラー: エンティティからスキーマ情報を抽出できませんでした");
        process::exit(1);
    }

    println!("✓ {}個のエンティティを検出しました\n", entity_schemas.len());

    // スキーマごとにテーブルをグループ化
    let mut schema_groups: HashMap<String, Vec<String>> = HashMap::new();
    for (table, schema) in &entity_schemas {
        schema_groups
            .entry(schema.clone())
            .or_insert_with(Vec::new)
            .push(table.clone());
    }

    // レポート表示
    println!("=== スキーマ別テーブル一覧 ===\n");
    let mut schemas: Vec<_> = schema_groups.keys().cloned().collect();
    schemas.sort();

    for schema in &schemas {
        if let Some(tables) = schema_groups.get(schema) {
            println!("📁 {} スキーマ ({} テーブル):", schema, tables.len());
            let mut sorted_tables = tables.clone();
            sorted_tables.sort();
            for table in sorted_tables {
                println!("   - {}", table);
            }
            println!();
        }
    }

    // 整合性チェック: notification_rel_* テーブルのスキーマを確認
    println!("=== 整合性チェック ===\n");
    let mut has_issues = false;

    // notification_rel_* テーブルは worker スキーマである必要がある
    for (table, schema) in &entity_schemas {
        if table.starts_with("notification_rel_") && schema != "worker" {
            eprintln!(
                "❌ エラー: テーブル '{}' はworkerスキーマである必要がありますが、'{}' スキーマになっています",
                table, schema
            );
            has_issues = true;
        }
    }

    // guild_* テーブルは guild_master スキーマである必要がある
    // ただし、guild_last_process_times はワーカーの実行状態を保存するため worker スキーマに配置
    for (table, schema) in &entity_schemas {
        if table.starts_with("guild_")
            && schema != "guild_master"
            && table != "guild_last_process_times" // 例外: ワーカー状態管理テーブル
        {
            eprintln!(
                "❌ エラー: テーブル '{}' はguild_masterスキーマである必要がありますが、'{}' スキーマになっています",
                table, schema
            );
            has_issues = true;
        }
    }

    // scheduled_task_* テーブルは worker スキーマである必要がある
    for (table, schema) in &entity_schemas {
        if table.starts_with("scheduled_task_") && schema != "worker" {
            eprintln!(
                "❌ エラー: テーブル '{}' はworkerスキーマである必要がありますが、'{}' スキーマになっています",
                table, schema
            );
            has_issues = true;
        }
    }

    if !has_issues {
        println!("✓ 整合性チェック完了: 問題は見つかりませんでした");
    } else {
        println!("\n❌ 整合性チェック失敗: 問題が見つかりました");
        process::exit(1);
    }

    println!("\n=== 検証完了 ===");
}
