/// データ変換サービス
///
/// PostgreSQLデータ型とスプレッドシート文字列の相互変換を行います。
/// 設計書: docs/develop/design/spreadsheet/data_conversion.md
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::errors::ValidationError;

/// データ変換サービストレイト
#[async_trait]
pub trait DataConverterServiceTrait: Send + Sync {
    /// PostgreSQL値をスプレッドシート文字列に変換
    fn to_spreadsheet_string(&self, value: &PostgresValue) -> Result<String, ValidationError>;

    /// スプレッドシート文字列をPostgreSQL値に変換
    #[allow(clippy::wrong_self_convention)]
    fn from_spreadsheet_string(
        &self,
        value: &str,
        target_type: PostgresType,
        field_name: &str,
    ) -> Result<PostgresValue, ValidationError>;

    /// 複数の値を一括変換（エラーを収集）
    fn convert_row_to_postgres(
        &self,
        row: Vec<String>,
        schema: &[ColumnSchema],
    ) -> (Vec<PostgresValue>, Vec<ValidationError>);

    /// 複数の値を一括変換（エラーと生成されたUUIDを収集）
    fn convert_row_to_postgres_with_uuid_tracking(
        &self,
        row: Vec<String>,
        schema: &[ColumnSchema],
    ) -> (Vec<PostgresValue>, Vec<ValidationError>, Vec<(usize, Uuid)>);

    /// PostgreSQL行をスプレッドシート行に変換
    fn convert_row_to_spreadsheet(
        &self,
        values: Vec<PostgresValue>,
    ) -> Result<Vec<String>, ValidationError>;
}

/// PostgreSQLデータ型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostgresType {
    Integer,
    BigInt,
    Text,
    Varchar,
    Boolean,
    Timestamp,
    TimestampTz,
    Date,
    Uuid,
    Json,
    JsonB,
    IntegerArray,
    TextArray,
}

/// PostgreSQL値
#[derive(Debug, Clone, PartialEq)]
pub enum PostgresValue {
    Null,
    Integer(i32),
    BigInt(i64),
    Text(String),
    Boolean(bool),
    Timestamp(NaiveDateTime),
    TimestampTz(DateTime<Local>),
    Date(NaiveDate),
    Uuid(Uuid),
    Json(JsonValue),
    IntegerArray(Vec<i32>),
    TextArray(Vec<String>),
}

/// カラムスキーマ定義
#[derive(Debug, Clone)]
pub struct ColumnSchema {
    /// カラム名
    pub column_name: String,
    /// PostgreSQLデータ型
    pub postgres_type: PostgresType,
    /// NULL許容
    pub nullable: bool,
}

/// データ変換サービス実装
#[derive(Clone)]
pub struct DataConverterService;

impl DataConverterService {
    const TIMESTAMP_FORMATS: &'static [&'static str] = &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H",
        "%Y-%m-%dT%H",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%dT%H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y/%m/%dT%H:%M",
        "%Y/%m/%d %H",
        "%Y/%m/%dT%H",
    ];

    const DATE_ONLY_FORMATS: &'static [&'static str] = &["%Y-%m-%d", "%Y/%m/%d"];

    const SUPPORTED_TIMESTAMP_FORMAT_MESSAGE: &'static str = "YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY/MM/DD HH:MM[:SS], YYYY/MM/DD, YYYY-MM-DD, RFC3339";

    pub fn new() -> Self {
        Self
    }

    /// NULLチェック
    fn is_null_value(value: &str) -> bool {
        value.is_empty() || value.eq_ignore_ascii_case("null")
    }

    /// 整数型変換（i32）
    fn parse_integer(value: &str, field_name: &str) -> Result<i32, ValidationError> {
        value
            .parse::<i32>()
            .map_err(|_| ValidationError::TypeConversionError {
                field: field_name.to_string(),
                value: value.to_string(),
                expected_type: "integer (i32)".to_string(),
            })
    }

    /// 整数型変換（i64）
    fn parse_bigint(value: &str, field_name: &str) -> Result<i64, ValidationError> {
        value
            .parse::<i64>()
            .map_err(|_| ValidationError::TypeConversionError {
                field: field_name.to_string(),
                value: value.to_string(),
                expected_type: "bigint (i64)".to_string(),
            })
    }

    /// ブール型変換
    fn parse_boolean(value: &str, field_name: &str) -> Result<bool, ValidationError> {
        match value.to_lowercase().as_str() {
            "true" | "t" | "yes" | "y" | "1" => Ok(true),
            "false" | "f" | "no" | "n" | "0" => Ok(false),
            _ => Err(ValidationError::TypeConversionError {
                field: field_name.to_string(),
                value: value.to_string(),
                expected_type: "boolean (true/false)".to_string(),
            }),
        }
    }

    /// 日時型変換（Timestamp）
    fn parse_timestamp(value: &str, _field_name: &str) -> Result<NaiveDateTime, ValidationError> {
        let trimmed = value.trim();

        for format in Self::TIMESTAMP_FORMATS {
            if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(dt);
            }
        }

        for format in Self::DATE_ONLY_FORMATS {
            if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
                return date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                    ValidationError::DateTimeFormatError {
                        value: trimmed.to_string(),
                        supported_formats: Self::SUPPORTED_TIMESTAMP_FORMAT_MESSAGE.to_string(),
                    }
                });
            }
        }

        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(dt.naive_utc());
        }

        Err(ValidationError::DateTimeFormatError {
            value: trimmed.to_string(),
            supported_formats: Self::SUPPORTED_TIMESTAMP_FORMAT_MESSAGE.to_string(),
        })
    }

    /// 日時型変換（TimestampTz）
    fn parse_timestamptz(
        value: &str,
        field_name: &str,
    ) -> Result<DateTime<Local>, ValidationError> {
        // ISO 8601形式: "YYYY-MM-DD HH:MM:SS+09:00"
        DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|_| {
                // タイムゾーンなしの場合、ローカルタイムゾーンとして解釈
                Self::parse_timestamp(value, field_name)
                    .map(|naive| Local.from_local_datetime(&naive).single())
                    .and_then(|opt| {
                        opt.ok_or_else(|| ValidationError::DateTimeFormatError {
                            value: value.to_string(),
                            supported_formats: Self::SUPPORTED_TIMESTAMP_FORMAT_MESSAGE.to_string(),
                        })
                    })
            })
    }

    /// 日付型変換
    fn parse_date(value: &str, _field_name: &str) -> Result<NaiveDate, ValidationError> {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
            ValidationError::DateTimeFormatError {
                value: value.to_string(),
                supported_formats: "YYYY-MM-DD".to_string(),
            }
        })
    }

    /// UUID型変換
    fn parse_uuid(value: &str, _field_name: &str) -> Result<Uuid, ValidationError> {
        Uuid::parse_str(value).map_err(|_| ValidationError::UuidFormatError {
            value: value.to_string(),
        })
    }

    /// JSON型変換
    fn parse_json(value: &str, field_name: &str) -> Result<JsonValue, ValidationError> {
        serde_json::from_str(value).map_err(|_| ValidationError::TypeConversionError {
            field: field_name.to_string(),
            value: value.to_string(),
            expected_type: "JSON".to_string(),
        })
    }

    /// 整数配列変換（カンマ区切り）
    fn parse_integer_array(value: &str, field_name: &str) -> Result<Vec<i32>, ValidationError> {
        if value.is_empty() {
            return Ok(Vec::new());
        }

        value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<i32>()
                    .map_err(|_| ValidationError::TypeConversionError {
                        field: field_name.to_string(),
                        value: s.to_string(),
                        expected_type: "integer array (comma-separated i32)".to_string(),
                    })
            })
            .collect()
    }

    /// 文字列配列変換（カンマ区切り）
    fn parse_text_array(value: &str, _field_name: &str) -> Result<Vec<String>, ValidationError> {
        if value.is_empty() {
            return Ok(Vec::new());
        }

        Ok(value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
}

impl Default for DataConverterService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataConverterServiceTrait for DataConverterService {
    fn to_spreadsheet_string(&self, value: &PostgresValue) -> Result<String, ValidationError> {
        Ok(match value {
            PostgresValue::Null => String::new(),
            PostgresValue::Integer(v) => v.to_string(),
            PostgresValue::BigInt(v) => v.to_string(),
            PostgresValue::Text(v) => v.clone(),
            PostgresValue::Boolean(v) => v.to_string(),
            PostgresValue::Timestamp(v) => v.format("%Y-%m-%d %H:%M:%S").to_string(),
            PostgresValue::TimestampTz(v) => v.to_rfc3339(),
            PostgresValue::Date(v) => v.format("%Y-%m-%d").to_string(),
            PostgresValue::Uuid(v) => v.to_string(),
            PostgresValue::Json(v) => {
                serde_json::to_string(v).map_err(|_| ValidationError::TypeConversionError {
                    field: "json".to_string(),
                    value: format!("{v:?}"),
                    expected_type: "JSON string".to_string(),
                })?
            }
            PostgresValue::IntegerArray(v) => v
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(","),
            PostgresValue::TextArray(v) => v.join(","),
        })
    }

    fn from_spreadsheet_string(
        &self,
        value: &str,
        target_type: PostgresType,
        field_name: &str,
    ) -> Result<PostgresValue, ValidationError> {
        // NULL値チェック（UUID型の場合は空文字列でも自動生成するためスキップ）
        if Self::is_null_value(value) && target_type != PostgresType::Uuid {
            return Ok(PostgresValue::Null);
        }

        match target_type {
            PostgresType::Integer => {
                Self::parse_integer(value, field_name).map(PostgresValue::Integer)
            }
            PostgresType::BigInt => {
                Self::parse_bigint(value, field_name).map(PostgresValue::BigInt)
            }
            PostgresType::Text | PostgresType::Varchar => {
                Ok(PostgresValue::Text(value.to_string()))
            }
            PostgresType::Boolean => {
                Self::parse_boolean(value, field_name).map(PostgresValue::Boolean)
            }
            PostgresType::Timestamp => {
                Self::parse_timestamp(value, field_name).map(PostgresValue::Timestamp)
            }
            PostgresType::TimestampTz => {
                Self::parse_timestamptz(value, field_name).map(PostgresValue::TimestampTz)
            }
            PostgresType::Date => Self::parse_date(value, field_name).map(PostgresValue::Date),
            PostgresType::Uuid => {
                // UUID型: 空文字列の場合は新規UUIDを生成
                if Self::is_null_value(value) {
                    Ok(PostgresValue::Uuid(Uuid::new_v4()))
                } else {
                    Self::parse_uuid(value, field_name).map(PostgresValue::Uuid)
                }
            }
            PostgresType::Json | PostgresType::JsonB => {
                Self::parse_json(value, field_name).map(PostgresValue::Json)
            }
            PostgresType::IntegerArray => {
                Self::parse_integer_array(value, field_name).map(PostgresValue::IntegerArray)
            }
            PostgresType::TextArray => {
                Self::parse_text_array(value, field_name).map(PostgresValue::TextArray)
            }
        }
    }

    fn convert_row_to_postgres(
        &self,
        row: Vec<String>,
        schema: &[ColumnSchema],
    ) -> (Vec<PostgresValue>, Vec<ValidationError>) {
        let (converted, errors, _) = self.convert_row_to_postgres_with_uuid_tracking(row, schema);
        (converted, errors)
    }

    fn convert_row_to_postgres_with_uuid_tracking(
        &self,
        row: Vec<String>,
        schema: &[ColumnSchema],
    ) -> (Vec<PostgresValue>, Vec<ValidationError>, Vec<(usize, Uuid)>) {
        let mut converted = Vec::new();
        let mut errors = Vec::new();
        let mut generated_uuids = Vec::new();

        for (index, column) in schema.iter().enumerate() {
            let value = row.get(index).map(|s| s.as_str()).unwrap_or("");
            let trimmed_value = value.trim();

            match self.from_spreadsheet_string(
                value,
                column.postgres_type.clone(),
                &column.column_name,
            ) {
                Ok(PostgresValue::Null) if !column.nullable => {
                    if trimmed_value.is_empty()
                        && matches!(
                            column.postgres_type,
                            PostgresType::Text | PostgresType::Varchar
                        )
                    {
                        converted.push(PostgresValue::Text(String::new()));
                    } else {
                        errors.push(ValidationError::RequiredFieldMissing {
                            field: column.column_name.clone(),
                        });
                        converted.push(PostgresValue::Null);
                    }
                }
                Ok(postgres_value) => {
                    // UUID型で値が空だった場合、自動生成されたUUIDを記録
                    if column.postgres_type == PostgresType::Uuid && Self::is_null_value(value) {
                        if let PostgresValue::Uuid(uuid) = &postgres_value {
                            generated_uuids.push((index, *uuid));
                        }
                    }
                    converted.push(postgres_value);
                }
                Err(e) => {
                    tracing::warn!(
                        column = %column.column_name,
                        error = %e,
                        "データ変換に失敗しました（NULLとして処理）"
                    );
                    errors.push(e);
                    converted.push(PostgresValue::Null);
                }
            }
        }

        (converted, errors, generated_uuids)
    }

    fn convert_row_to_spreadsheet(
        &self,
        values: Vec<PostgresValue>,
    ) -> Result<Vec<String>, ValidationError> {
        values
            .iter()
            .map(|v| self.to_spreadsheet_string(v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_null_value() {
        assert!(DataConverterService::is_null_value(""));
        assert!(DataConverterService::is_null_value("null"));
        assert!(DataConverterService::is_null_value("NULL"));
        assert!(DataConverterService::is_null_value("Null"));
        assert!(!DataConverterService::is_null_value("0"));
        assert!(!DataConverterService::is_null_value("false"));
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            DataConverterService::parse_integer("123", "test_field").unwrap(),
            123
        );
        assert_eq!(
            DataConverterService::parse_integer("-456", "test_field").unwrap(),
            -456
        );
        assert!(DataConverterService::parse_integer("abc", "test_field").is_err());
        assert!(DataConverterService::parse_integer("12.34", "test_field").is_err());
    }

    #[test]
    fn test_parse_boolean() {
        assert!(DataConverterService::parse_boolean("true", "test_field").unwrap());
        assert!(DataConverterService::parse_boolean("TRUE", "test_field").unwrap());
        assert!(DataConverterService::parse_boolean("1", "test_field").unwrap());
        assert!(!DataConverterService::parse_boolean("false", "test_field").unwrap());
        assert!(!DataConverterService::parse_boolean("0", "test_field").unwrap());
        assert!(DataConverterService::parse_boolean("maybe", "test_field").is_err());
    }

    #[test]
    fn test_parse_timestamp() {
        let result = DataConverterService::parse_timestamp("2024-01-15 12:34:56", "test_field");
        assert!(result.is_ok());

        let result = DataConverterService::parse_timestamp("2024-01-15T12:34:56", "test_field");
        assert!(result.is_ok());

        assert!(DataConverterService::parse_timestamp("invalid", "test_field").is_err());
    }

    #[test]
    fn test_parse_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = DataConverterService::parse_uuid(uuid_str, "test_field");
        assert!(result.is_ok());

        assert!(DataConverterService::parse_uuid("invalid-uuid", "test_field").is_err());
    }

    #[test]
    fn test_parse_integer_array() {
        let result = DataConverterService::parse_integer_array("1,2,3", "test_field").unwrap();
        assert_eq!(result, vec![1, 2, 3]);

        let result = DataConverterService::parse_integer_array("10, 20, 30", "test_field").unwrap();
        assert_eq!(result, vec![10, 20, 30]);

        let result = DataConverterService::parse_integer_array("", "test_field").unwrap();
        assert_eq!(result, Vec::<i32>::new());

        assert!(DataConverterService::parse_integer_array("1,abc,3", "test_field").is_err());
    }

    #[test]
    fn test_parse_text_array() {
        let result = DataConverterService::parse_text_array("a,b,c", "test_field").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);

        let result = DataConverterService::parse_text_array("foo, bar, baz", "test_field").unwrap();
        assert_eq!(result, vec!["foo", "bar", "baz"]);

        let result = DataConverterService::parse_text_array("", "test_field").unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[tokio::test]
    async fn test_to_spreadsheet_string() {
        let service = DataConverterService::new();

        assert_eq!(
            service.to_spreadsheet_string(&PostgresValue::Null).unwrap(),
            ""
        );
        assert_eq!(
            service
                .to_spreadsheet_string(&PostgresValue::Integer(123))
                .unwrap(),
            "123"
        );
        assert_eq!(
            service
                .to_spreadsheet_string(&PostgresValue::Text("hello".to_string()))
                .unwrap(),
            "hello"
        );
        assert_eq!(
            service
                .to_spreadsheet_string(&PostgresValue::Boolean(true))
                .unwrap(),
            "true"
        );
        assert_eq!(
            service
                .to_spreadsheet_string(&PostgresValue::IntegerArray(vec![1, 2, 3]))
                .unwrap(),
            "1,2,3"
        );
    }

    #[tokio::test]
    async fn test_from_spreadsheet_string() {
        let service = DataConverterService::new();

        // Integer
        let result = service
            .from_spreadsheet_string("123", PostgresType::Integer, "test_field")
            .unwrap();
        assert_eq!(result, PostgresValue::Integer(123));

        // Text
        let result = service
            .from_spreadsheet_string("hello", PostgresType::Text, "test_field")
            .unwrap();
        assert_eq!(result, PostgresValue::Text("hello".to_string()));

        // Boolean
        let result = service
            .from_spreadsheet_string("true", PostgresType::Boolean, "test_field")
            .unwrap();
        assert_eq!(result, PostgresValue::Boolean(true));

        // NULL
        let result = service
            .from_spreadsheet_string("", PostgresType::Integer, "test_field")
            .unwrap();
        assert_eq!(result, PostgresValue::Null);

        // IntegerArray
        let result = service
            .from_spreadsheet_string("1,2,3", PostgresType::IntegerArray, "test_field")
            .unwrap();
        assert_eq!(result, PostgresValue::IntegerArray(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_convert_row_to_postgres() {
        let service = DataConverterService::new();

        let schema = vec![
            ColumnSchema {
                column_name: "id".to_string(),
                postgres_type: PostgresType::Integer,
                nullable: false,
            },
            ColumnSchema {
                column_name: "name".to_string(),
                postgres_type: PostgresType::Text,
                nullable: false,
            },
            ColumnSchema {
                column_name: "age".to_string(),
                postgres_type: PostgresType::Integer,
                nullable: true,
            },
        ];

        let row = vec!["1".to_string(), "Alice".to_string(), "30".to_string()];

        let (converted, errors) = service.convert_row_to_postgres(row, &schema);

        assert_eq!(errors.len(), 0);
        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0], PostgresValue::Integer(1));
        assert_eq!(converted[1], PostgresValue::Text("Alice".to_string()));
        assert_eq!(converted[2], PostgresValue::Integer(30));
    }

    #[tokio::test]
    async fn test_convert_row_to_postgres_with_errors() {
        let service = DataConverterService::new();

        let schema = vec![
            ColumnSchema {
                column_name: "id".to_string(),
                postgres_type: PostgresType::Integer,
                nullable: false,
            },
            ColumnSchema {
                column_name: "name".to_string(),
                postgres_type: PostgresType::Text,
                nullable: false,
            },
        ];

        // idが不正な値
        let row = vec!["invalid".to_string(), "Alice".to_string()];

        let (converted, errors) = service.convert_row_to_postgres(row, &schema);

        assert_eq!(errors.len(), 1);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0], PostgresValue::Null); // エラー時はNULL
        assert_eq!(converted[1], PostgresValue::Text("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_convert_row_to_spreadsheet() {
        let service = DataConverterService::new();

        let values = vec![
            PostgresValue::Integer(1),
            PostgresValue::Text("Alice".to_string()),
            PostgresValue::Boolean(true),
            PostgresValue::Null,
        ];

        let result = service.convert_row_to_spreadsheet(values).unwrap();

        assert_eq!(result, vec!["1", "Alice", "true", ""]);
    }
}
