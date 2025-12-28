use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// エンティティファイルからスキーマ情報を抽出
fn extract_schema_info() -> HashMap<String, String> {
    let mut schema_map = HashMap::new();
    let entities_dir = Path::new("src/models/entities");

    if !entities_dir.exists() {
        eprintln!("警告: エンティティディレクトリが見つかりません: {entities_dir:?}");
        return schema_map;
    }

    // 再帰的にエンティティファイルを探索
    extract_schema_info_recursive(entities_dir, &mut schema_map);

    schema_map
}

/// エンティティディレクトリを再帰的に探索してスキーマ情報を抽出
fn extract_schema_info_recursive(dir: &Path, schema_map: &mut HashMap<String, String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("ディレクトリの読み取りに失敗: {dir:?}, error: {e}");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // サブディレクトリの場合は再帰的に探索
        if path.is_dir() {
            extract_schema_info_recursive(&path, schema_map);
            continue;
        }

        // .rsファイル以外はスキップ
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        // mod.rsはスキップ
        if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("ファイル読み取りエラー {path:?}: {e}");
                continue;
            }
        };

        // #[sea_orm(schema_name = "...", table_name = "...")] パターンを探す
        if let Some((schema_name, table_name)) = parse_entity_file(&content) {
            schema_map.insert(table_name, schema_name);
        }
    }
}

/// エンティティファイルの内容からschema_nameとtable_nameを抽出
fn parse_entity_file(content: &str) -> Option<(String, String)> {
    let mut schema_name = None;
    let mut table_name = None;
    let mut in_sea_orm_attr = false;

    // sea_orm属性を含む行を探す（複数行対応）
    for line in content.lines() {
        let trimmed = line.trim();

        // #[sea_orm( で属性開始
        if trimmed.starts_with("#[sea_orm(") {
            in_sea_orm_attr = true;
        }

        // 属性内またはインライン属性の場合
        if in_sea_orm_attr || trimmed.starts_with("#[sea_orm(") {
            // schema_name を抽出
            if let Some(start) = trimmed.find("schema_name = \"") {
                let start_idx = start + "schema_name = \"".len();
                if let Some(end_idx) = trimmed[start_idx..].find('"') {
                    schema_name = Some(trimmed[start_idx..start_idx + end_idx].to_string());
                }
            }

            // table_name を抽出
            if let Some(start) = trimmed.find("table_name = \"") {
                let start_idx = start + "table_name = \"".len();
                if let Some(end_idx) = trimmed[start_idx..].find('"') {
                    table_name = Some(trimmed[start_idx..start_idx + end_idx].to_string());
                }
            }

            // )] で属性終了
            if trimmed.ends_with(")]") {
                in_sea_orm_attr = false;
                // 両方見つかっていれば早期リターン
                if schema_name.is_some() && table_name.is_some() {
                    break;
                }
            }
        }
    }

    match (schema_name, table_name) {
        (Some(s), Some(t)) => Some((s, t)),
        _ => None,
    }
}

/// get_schema_name関数のコードを生成
fn generate_schema_name_function(schema_map: &HashMap<String, String>) -> String {
    // スキーマごとにテーブルをグループ化
    let mut schema_groups: HashMap<String, Vec<String>> = HashMap::new();
    for (table, schema) in schema_map {
        schema_groups
            .entry(schema.clone())
            .or_default()
            .push(table.clone());
    }

    // 各スキーマグループをソート
    for tables in schema_groups.values_mut() {
        tables.sort();
    }

    let mut code = String::from(
        r#"use sea_orm::sea_query::{Alias, IntoIden, TableRef};

/// テーブル名からスキーマ名を取得
///
/// テーブル名から適切なスキーマ名を返します。
/// この関数はエンティティの`#[sea_orm(schema_name = "...")]`属性と同期しています。
///
/// # 注意
/// この関数の定義はビルド時に自動生成されます。
/// 手動で編集しないでください。エンティティ定義を変更すると自動的に更新されます。
#[allow(dead_code)]
pub fn get_schema_name(table_name: &str) -> &str {
    match table_name {
"#,
    );

    // スキーマ名でソート
    let mut schemas: Vec<_> = schema_groups.keys().cloned().collect();
    schemas.sort();

    for schema in schemas {
        if let Some(tables) = schema_groups.get(&schema) {
            code.push_str(&format!("        // {schema} スキーマ\n"));

            for (i, table) in tables.iter().enumerate() {
                if i == 0 {
                    code.push_str(&format!("        \"{table}\"\n"));
                } else {
                    code.push_str(&format!("        | \"{table}\"\n"));
                }
            }
            code.push_str(&format!(" => \"{schema}\",\n"));
        }
    }

    code.push_str(
        r#"        // デフォルトはpublicスキーマ（後方互換性のため）
        _ => "public",
    }
}

/// テーブル名からスキーマ修飾されたTableRefを取得
///
/// スキーマ名とテーブル名を使用して、適切なTableRefを返します。
#[allow(dead_code)]
pub fn get_entity_table_ref(table_name: &str) -> TableRef {
    let schema = get_schema_name(table_name);
    // スキーマがpublicでない場合は、スキーマ修飾したTableRefを返す
    if schema != "public" {
        TableRef::SchemaTable(
            Alias::new(schema).into_iden(),
            Alias::new(table_name).into_iden(),
        )
    } else {
        TableRef::Table(Alias::new(table_name).into_iden())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tables_have_schema() {
        // この部分は自動生成されたテストです
        // 全てのテーブルがスキーマを持っていることを確認
"#,
    );

    // テストケースを生成
    let mut all_tables: Vec<_> = schema_map.keys().cloned().collect();
    all_tables.sort();

    for table in all_tables {
        if let Some(expected_schema) = schema_map.get(&table) {
            code.push_str(&format!(
                "        assert_eq!(get_schema_name(\"{table}\"), \"{expected_schema}\");\n"
            ));
        }
    }

    code.push_str(
        r#"    }

    #[test]
    fn test_public_schema_default() {
        assert_eq!(get_schema_name("unknown_table"), "public");
    }

    #[test]
    fn test_get_entity_table_ref() {
        // 関数が正常に実行できることを確認
        let table_ref = get_entity_table_ref("unknown_table");
        assert!(matches!(table_ref, TableRef::Table(_)));
    }
}
"#,
    );

    code
}

fn main() {
    println!("cargo:rerun-if-changed=src/models/entities/");
    println!("cargo:rerun-if-changed=locales/messages.yml");

    let schema_map = extract_schema_info();

    if schema_map.is_empty() {
        eprintln!("警告: エンティティからスキーマ情報を抽出できませんでした");
        return;
    }

    println!("抽出されたスキーマ情報:");
    for (table, schema) in &schema_map {
        println!("  {table} -> {schema}");
    }

    let generated_code = generate_schema_name_function(&schema_map);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR環境変数が設定されていません");
    let dest_path = Path::new(&out_dir).join("generated_schema_utils.rs");

    fs::write(&dest_path, generated_code).expect("生成されたコードの書き込みに失敗しました");

    println!("スキーマユーティリティコードを生成しました: {dest_path:?}");
}
