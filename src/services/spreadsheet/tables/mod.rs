/// テーブル固有設定モジュール
///
/// 各テーブルのスプレッドシート読み書き設定を個別ファイルで管理します。
use std::collections::HashMap;

use sea_orm::{ColumnTrait, ColumnType, EntityTrait, Iden, Iterable};

use crate::services::spreadsheet::{ColumnSchema, PostgresType};

mod battle_types;
mod channel_types;
mod elements;
mod environments;
mod event_schedule_details;
mod event_schedules;
mod message_texts;
mod quest_aliases;
mod quests;

pub use battle_types::BattleTypesTable;
pub use channel_types::ChannelTypesTable;
pub use elements::ElementsTable;
pub use environments::EnvironmentsTable;
pub use event_schedule_details::EventScheduleDetailsTable;
pub use event_schedules::EventSchedulesTable;
pub use message_texts::MessageTextsTable;
pub use quest_aliases::QuestAliasesTable;
pub use quests::QuestsTable;

/// テーブル登録情報
#[derive(Clone)]
pub struct TableRegistration {
    pub table_name: &'static str,
    pub aliases: Vec<&'static str>,
    pub schema: Vec<ColumnSchema>,
}

impl TableRegistration {
    fn new<T: TableConfig>() -> Self {
        Self {
            table_name: T::table_name(),
            aliases: T::table_aliases(),
            schema: T::read_columns(),
        }
    }
}

/// 全テーブルの登録情報を取得
pub fn table_registrations() -> Vec<TableRegistration> {
    vec![
        TableRegistration::new::<BattleTypesTable>(),
        TableRegistration::new::<ChannelTypesTable>(),
        TableRegistration::new::<EnvironmentsTable>(),
        TableRegistration::new::<EventSchedulesTable>(),
        TableRegistration::new::<EventScheduleDetailsTable>(),
        TableRegistration::new::<ElementsTable>(),
        TableRegistration::new::<MessageTextsTable>(),
        TableRegistration::new::<QuestsTable>(),
        TableRegistration::new::<QuestAliasesTable>(),
    ]
}

/// テーブル固有設定トレイト
///
/// 各テーブルのスプレッドシート読み書き設定を定義します。
pub trait TableConfig: Send + Sync {
    /// エンティティ型
    type Entity: EntityTrait;

    /// テーブル名（データベース側）
    fn table_name() -> &'static str;

    /// スプレッドシート上の別名一覧（オプション）
    fn table_aliases() -> Vec<&'static str> {
        vec![]
    }

    /// 読み込み時に除外する列（デフォルト: created_at, updated_at）
    /// スプレッドシート側にはこれらの列が存在しないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["created_at", "updated_at"]
    }

    /// 書き込み時に除外する列（デフォルト: created_at, updated_at）
    /// スプレッドシート側にはこれらの列が存在しないため除外
    fn excluded_columns_for_write() -> Vec<&'static str> {
        vec!["created_at", "updated_at"]
    }

    /// 読み込み対象列の取得（自動抽出）
    fn read_columns() -> Vec<ColumnSchema> {
        let all_columns = extract_entity_schema::<Self::Entity>(Self::table_name());
        let excluded = Self::excluded_columns_for_read();

        all_columns
            .into_iter()
            .filter(|col| !excluded.contains(&col.column_name.as_str()))
            .collect()
    }

    /// 書き込み対象列の取得（自動抽出）
    fn write_columns() -> Vec<ColumnSchema> {
        let all_columns = extract_entity_schema::<Self::Entity>(Self::table_name());
        let excluded = Self::excluded_columns_for_write();

        all_columns
            .into_iter()
            .filter(|col| !excluded.contains(&col.column_name.as_str()))
            .collect()
    }
}

/// エンティティからスキーマを抽出するヘルパー関数
///
/// SchemaExtractorServiceから移植した共通処理
fn extract_entity_schema<E>(table_name: &str) -> Vec<ColumnSchema>
where
    E: EntityTrait,
{
    let mut schemas = Vec::new();

    // エンティティの全カラムを取得
    for column in E::Column::iter() {
        let column_def = column.def();
        let column_name = column.to_string();

        // ColumnTypeを取得してPostgresTypeに変換
        if let Some(postgres_type) = convert_column_type(&column_def.get_column_type()) {
            let is_nullable = column_def.is_null();

            schemas.push(ColumnSchema {
                column_name,
                postgres_type,
                nullable: is_nullable,
            });
        }
    }

    schemas
}

/// SeaORMのColumnTypeをPostgresTypeに変換
fn convert_column_type(column_type: &ColumnType) -> Option<PostgresType> {
    match column_type {
        ColumnType::Integer => Some(PostgresType::Integer),
        ColumnType::BigInteger => Some(PostgresType::BigInt),
        ColumnType::String(_) | ColumnType::Char(_) => Some(PostgresType::Text),
        ColumnType::Text => Some(PostgresType::Text),
        ColumnType::Boolean => Some(PostgresType::Boolean),
        ColumnType::DateTime => Some(PostgresType::Timestamp),
        ColumnType::TimestampWithTimeZone => Some(PostgresType::TimestampTz),
        ColumnType::Date => Some(PostgresType::Date),
        ColumnType::Uuid => Some(PostgresType::Uuid),
        ColumnType::Json => Some(PostgresType::Json),
        ColumnType::JsonBinary => Some(PostgresType::JsonB),
        ColumnType::Array(_) => None, // 未サポート
        _ => None,
    }
}

/// 全テーブル設定を登録
///
/// テーブル名（および別名）をキーとして、スキーマ情報を格納したHashMapを返します。
pub fn register_all_tables() -> HashMap<String, Vec<ColumnSchema>> {
    let mut schemas: HashMap<String, Vec<ColumnSchema>> = HashMap::new();

    for registration in table_registrations() {
        schemas.insert(
            registration.table_name.to_string(),
            registration.schema.clone(),
        );

        for alias in registration.aliases {
            schemas.insert(alias.to_string(), registration.schema.clone());
        }
    }

    schemas
}

/// テーブル名（または別名）からスキーマを取得
pub fn get_table_schema(table_name: &str) -> Option<Vec<ColumnSchema>> {
    let all_tables = register_all_tables();
    all_tables.get(table_name).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_tables() {
        let schemas = register_all_tables();

        // 登録した全テーブルが含まれているか確認
        assert!(schemas.contains_key("battle_types"));
        assert!(schemas.contains_key("environments"));
        assert!(schemas.contains_key("event_schedules"));
        assert!(schemas.contains_key("event_schedule_details"));
        assert!(schemas.contains_key("message_texts"));
        assert!(schemas.contains_key("quests"));
        assert!(schemas.contains_key("quest_aliases"));
    }

    #[test]
    fn test_table_aliases() {
        let schemas = register_all_tables();

        // 別名でも取得できるか確認
        assert!(schemas.contains_key("messages")); // message_textsの別名
        assert!(schemas.contains_key("quests_alias")); // quest_aliasesの別名
    }

    #[test]
    fn test_get_table_schema() {
        // 本名で取得
        let schema = get_table_schema("quests");
        assert!(schema.is_some());

        // 別名で取得
        let schema = get_table_schema("messages");
        assert!(schema.is_some());

        // 未登録のテーブル
        let schema = get_table_schema("unknown_table");
        assert!(schema.is_none());
    }

    #[test]
    fn test_excluded_columns() {
        let schemas = register_all_tables();

        // 全テーブルでcreated_atとupdated_atが除外されていることを確認
        for (table_name, schema) in schemas.iter() {
            assert!(
                !schema.iter().any(|col| col.column_name == "created_at"),
                "テーブル「{}」でcreated_atが除外されていません",
                table_name
            );
            assert!(
                !schema.iter().any(|col| col.column_name == "updated_at"),
                "テーブル「{}」でupdated_atが除外されていません",
                table_name
            );
        }
    }

    #[test]
    fn test_table_config_read_columns() {
        // BattleTypesTableで除外動作を確認
        let columns = BattleTypesTable::read_columns();

        // created_atとupdated_atが含まれていないこと
        assert!(!columns.iter().any(|col| col.column_name == "created_at"));
        assert!(!columns.iter().any(|col| col.column_name == "updated_at"));

        // それ以外のカラムは含まれていること
        assert!(columns.iter().any(|col| col.column_name == "id"));
        assert!(columns.iter().any(|col| col.column_name == "name"));
    }
}
