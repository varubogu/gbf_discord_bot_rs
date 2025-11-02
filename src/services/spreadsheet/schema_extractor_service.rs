/// スキーマ抽出サービス
///
/// SeaORMエンティティからテーブルスキーマ情報を抽出し、
/// スプレッドシート取り込み用のColumnSchemaに変換します。
use std::collections::HashMap;

use tracing::info;

use crate::services::spreadsheet::ColumnSchema;
use crate::services::spreadsheet::tables;

/// 登録済みテーブルのスキーマ情報
#[derive(Debug, Clone)]
pub struct RegisteredTableSchema {
    /// テーブル名（本名）
    pub table_name: String,
    /// 参照可能な別名一覧
    pub aliases: Vec<String>,
    /// カラムスキーマ一覧
    pub schema: Vec<ColumnSchema>,
}

/// スキーマ抽出サービストレイト
pub trait SchemaExtractorServiceTrait: Send + Sync {
    /// 全テーブルのスキーマをHashMapで取得
    fn extract_all_schemas(&self) -> HashMap<String, Vec<ColumnSchema>>;

    /// 特定のテーブルのスキーマを取得
    fn extract_schema(&self, table_name: &str) -> Option<Vec<ColumnSchema>>;

    /// 登録済みテーブルのメタ情報を取得（別名含む）
    fn extract_registered_tables(&self) -> Vec<RegisteredTableSchema>;
}

/// スキーマ抽出サービス実装
pub struct SchemaExtractorService;

impl SchemaExtractorService {
    pub fn new() -> Self {
        Self
    }
}

impl SchemaExtractorServiceTrait for SchemaExtractorService {
    fn extract_all_schemas(&self) -> HashMap<String, Vec<ColumnSchema>> {
        let schemas = tables::register_all_tables();

        info!(
            schema_count = schemas.len(),
            "全テーブルのスキーマ抽出が完了しました"
        );

        schemas
    }

    fn extract_schema(&self, table_name: &str) -> Option<Vec<ColumnSchema>> {
        tables::get_table_schema(table_name)
    }

    fn extract_registered_tables(&self) -> Vec<RegisteredTableSchema> {
        tables::table_registrations()
            .into_iter()
            .map(|registration| RegisteredTableSchema {
                table_name: registration.table_name.to_string(),
                aliases: registration
                    .aliases
                    .into_iter()
                    .map(|alias| alias.to_string())
                    .collect(),
                schema: registration.schema,
            })
            .collect()
    }
}

impl Default for SchemaExtractorService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quests_schema() {
        let service = SchemaExtractorService::new();
        let schema = service.extract_schema("quests");

        assert!(schema.is_some());
        let schema = schema.unwrap();

        // questsテーブルには最低限これらのカラムがあるはず
        assert!(schema.iter().any(|c| c.column_name == "id"));
        assert!(schema.iter().any(|c| c.column_name == "name"));
        assert!(
            schema
                .iter()
                .any(|c| c.column_name == "default_battle_style")
        );
    }

    #[test]
    fn test_extract_all_schemas() {
        let service = SchemaExtractorService::new();
        let schemas = service.extract_all_schemas();

        // 登録した全テーブルが含まれているか確認
        assert!(schemas.contains_key("battle_types"));
        assert!(schemas.contains_key("environments"));
        assert!(schemas.contains_key("quests"));
        assert!(schemas.contains_key("quest_aliases"));
    }

    #[test]
    fn test_extract_unknown_table() {
        let service = SchemaExtractorService::new();
        let schema = service.extract_schema("unknown_table");

        assert!(schema.is_none());
    }

    #[test]
    fn test_table_aliases() {
        let service = SchemaExtractorService::new();

        // 別名でも取得できるか確認
        let schema_messages = service.extract_schema("messages");
        assert!(schema_messages.is_some());

        let schema_quests_alias = service.extract_schema("quests_alias");
        assert!(schema_quests_alias.is_some());
    }

    #[test]
    fn test_extract_registered_tables_contains_aliases() {
        let service = SchemaExtractorService::new();
        let registered = service.extract_registered_tables();

        assert!(!registered.is_empty());

        let message_texts = registered
            .iter()
            .find(|table| table.table_name == "message_texts")
            .expect("message_texts table should be registered");

        assert!(message_texts.aliases.contains(&"messages".to_string()));
        assert!(!message_texts.schema.is_empty());
    }
}
