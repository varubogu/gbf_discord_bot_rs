/// スプレッドシート読み込みサービス
///
/// Google Sheetsからデータを読み込み、PostgreSQL用のデータ構造に変換します。
/// 設計書: docs/develop/design/spreadsheet/service_layer.md

use async_trait::async_trait;
use google_sheets4::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector, Sheets};

use crate::errors::{ExternalServiceError, ValidationError};
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterServiceTrait, PostgresValue, TableDefinition,
    TableDefinitionServiceTrait,
};

/// スプレッドシート読み込み結果
#[derive(Debug, Clone)]
pub struct ReadResult {
    /// テーブル名（英語）
    pub table_name: String,
    /// 読み込んだ行データ
    pub rows: Vec<RowData>,
    /// 読み込み時に発生したエラー
    pub errors: Vec<ReadError>,
}

/// 行データ
#[derive(Debug, Clone)]
pub struct RowData {
    /// 行番号（スプレッドシート上の行番号、1始まり）
    pub row_number: usize,
    /// 変換済みPostgreSQL値
    pub values: Vec<PostgresValue>,
}

/// 読み込みエラー
#[derive(Debug, Clone)]
pub struct ReadError {
    /// テーブル名
    pub table_name: String,
    /// 行番号
    pub row_number: usize,
    /// エラーメッセージ
    pub message: String,
}

/// スプレッドシート読み込みサービストレイト
#[async_trait]
pub trait SpreadsheetReaderServiceTrait: Send + Sync {
    /// スプレッドシートからテーブル定義を読み込み
    async fn read_table_definitions(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
    ) -> Result<Vec<TableDefinition>, ExternalServiceError>;

    /// 指定されたテーブルのデータを読み込み
    async fn read_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
        schema: &[ColumnSchema],
    ) -> Result<ReadResult, ExternalServiceError>;

    /// スプレッドシート全体を読み込み（全テーブル）
    async fn read_all_tables(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        schemas: &std::collections::HashMap<String, Vec<ColumnSchema>>,
    ) -> Result<Vec<ReadResult>, ExternalServiceError>;
}

/// スプレッドシート読み込みサービス実装
pub struct SpreadsheetReaderService<T, D>
where
    T: TableDefinitionServiceTrait,
    D: DataConverterServiceTrait,
{
    table_definition_service: T,
    data_converter_service: D,
}

impl<T, D> SpreadsheetReaderService<T, D>
where
    T: TableDefinitionServiceTrait,
    D: DataConverterServiceTrait,
{
    pub fn new(table_definition_service: T, data_converter_service: D) -> Self {
        Self {
            table_definition_service,
            data_converter_service,
        }
    }

    /// シート名からセル範囲を取得（A2以降、ヘッダー除く）
    fn build_range(sheet_name: &str) -> String {
        format!("'{}'!A2:ZZ", sheet_name)
    }

    /// 生の文字列行をPostgreSQL値に変換
    fn convert_raw_row(
        &self,
        raw_values: Vec<String>,
        schema: &[ColumnSchema],
        row_number: usize,
    ) -> (RowData, Vec<ValidationError>) {
        let (values, errors) = self
            .data_converter_service
            .convert_row_to_postgres(raw_values, schema);

        let row_data = RowData { row_number, values };

        (row_data, errors)
    }
}

#[async_trait]
impl<T, D> SpreadsheetReaderServiceTrait for SpreadsheetReaderService<T, D>
where
    T: TableDefinitionServiceTrait + Send + Sync,
    D: DataConverterServiceTrait + Send + Sync,
{
    async fn read_table_definitions(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
    ) -> Result<Vec<TableDefinition>, ExternalServiceError> {
        // 「テーブル名」シートを読み込み
        let range = "テーブル名!A2:D";

        let result = sheets_client
            .spreadsheets()
            .values_get(spreadsheet_id, range)
            .doit()
            .await
            .map_err(|e| ExternalServiceError::GoogleSheetsApiError {
                message: format!("テーブル定義シートの読み込みに失敗しました: {}", e),
            })?;

        // 値を取得
        let values = result
            .1
            .values
            .ok_or_else(|| ExternalServiceError::GoogleSheetsApiError {
                message: "テーブル定義シートにデータが存在しません".to_string(),
            })?;

        // 文字列の2次元配列に変換
        let string_values: Vec<Vec<String>> = values
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| cell.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .collect();

        // TableDefinitionServiceでパース
        self.table_definition_service
            .parse_table_definitions(string_values)
            .await
            .map_err(|e| ExternalServiceError::GoogleSheetsApiError {
                message: format!("テーブル定義のパースに失敗しました: {}", e),
            })
    }

    async fn read_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
        schema: &[ColumnSchema],
    ) -> Result<ReadResult, ExternalServiceError> {
        // シート名（日本語テーブル名）からセル範囲を構築
        let range = Self::build_range(&table_definition.table_name_jp);

        tracing::info!(
            table_name = %table_definition.table_name_en,
            sheet_name = %table_definition.table_name_jp,
            range = %range,
            "テーブルデータの読み込みを開始します"
        );

        // Google Sheets APIでデータを取得
        let result = sheets_client
            .spreadsheets()
            .values_get(spreadsheet_id, &range)
            .doit()
            .await
            .map_err(|e| ExternalServiceError::GoogleSheetsApiError {
                message: format!(
                    "シート「{}」の読み込みに失敗しました: {}",
                    table_definition.table_name_jp, e
                ),
            })?;

        // 値を取得
        let values = result.1.values.unwrap_or_default();

        if values.is_empty() {
            tracing::info!(
                table_name = %table_definition.table_name_en,
                "シートにデータが存在しません（空のテーブル）"
            );
            return Ok(ReadResult {
                table_name: table_definition.table_name_en.clone(),
                rows: Vec::new(),
                errors: Vec::new(),
            });
        }

        // 各行を変換
        let mut rows = Vec::new();
        let mut errors = Vec::new();

        for (index, row) in values.into_iter().enumerate() {
            // 行番号（スプレッドシート上は2行目から、ヘッダーが1行目）
            let row_number = index + 2;

            // JSON値を文字列に変換
            let raw_values: Vec<String> = row
                .into_iter()
                .map(|cell| cell.as_str().unwrap_or("").to_string())
                .collect();

            // スキーマに基づいて変換
            let (row_data, row_errors) = self.convert_raw_row(raw_values, schema, row_number);

            // エラーがあれば記録
            for error in row_errors {
                errors.push(ReadError {
                    table_name: table_definition.table_name_en.clone(),
                    row_number,
                    message: error.to_string(),
                });
            }

            rows.push(row_data);
        }

        tracing::info!(
            table_name = %table_definition.table_name_en,
            row_count = rows.len(),
            error_count = errors.len(),
            "テーブルデータの読み込みが完了しました"
        );

        Ok(ReadResult {
            table_name: table_definition.table_name_en.clone(),
            rows,
            errors,
        })
    }

    async fn read_all_tables(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        schemas: &std::collections::HashMap<String, Vec<ColumnSchema>>,
    ) -> Result<Vec<ReadResult>, ExternalServiceError> {
        // テーブル定義を読み込み
        let table_definitions = self
            .read_table_definitions(sheets_client, spreadsheet_id)
            .await?;

        tracing::info!(
            table_count = table_definitions.len(),
            "全テーブルの読み込みを開始します"
        );

        let mut results = Vec::new();

        // 各テーブルを順次読み込み
        for table_def in table_definitions {
            // スキーマを取得
            let schema = match schemas.get(&table_def.table_name_en) {
                Some(s) => s,
                None => {
                    tracing::warn!(
                        table_name = %table_def.table_name_en,
                        "スキーマが見つかりません。スキップします"
                    );
                    continue;
                }
            };

            // データを読み込み
            match self
                .read_table_data(sheets_client, spreadsheet_id, &table_def, schema)
                .await
            {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    tracing::error!(
                        table_name = %table_def.table_name_en,
                        error = %e,
                        "テーブルの読み込みに失敗しました"
                    );
                    // エラーがあっても他のテーブルは読み込み続行
                    results.push(ReadResult {
                        table_name: table_def.table_name_en.clone(),
                        rows: Vec::new(),
                        errors: vec![ReadError {
                            table_name: table_def.table_name_en.clone(),
                            row_number: 0,
                            message: e.to_string(),
                        }],
                    });
                }
            }
        }

        tracing::info!(
            total_results = results.len(),
            "全テーブルの読み込みが完了しました"
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::spreadsheet::{
        DataConverterService, PostgresType, TableDefinitionService, TableIO, TableType,
    };

    #[test]
    fn test_build_range() {
        let range = SpreadsheetReaderService::<TableDefinitionService, DataConverterService>::build_range("クエスト");
        assert_eq!(range, "'クエスト'!A2:ZZ");
    }

    #[tokio::test]
    async fn test_convert_raw_row() {
        let table_def_service = TableDefinitionService::new();
        let converter_service = DataConverterService::new();
        let reader_service =
            SpreadsheetReaderService::new(table_def_service, converter_service);

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

        let raw_values = vec!["123".to_string(), "テスト".to_string()];

        let (row_data, errors) = reader_service.convert_raw_row(raw_values, &schema, 2);

        assert_eq!(row_data.row_number, 2);
        assert_eq!(row_data.values.len(), 2);
        assert_eq!(row_data.values[0], PostgresValue::Integer(123));
        assert_eq!(row_data.values[1], PostgresValue::Text("テスト".to_string()));
        assert_eq!(errors.len(), 0);
    }

    #[tokio::test]
    async fn test_convert_raw_row_with_errors() {
        let table_def_service = TableDefinitionService::new();
        let converter_service = DataConverterService::new();
        let reader_service =
            SpreadsheetReaderService::new(table_def_service, converter_service);

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

        // 不正なデータ（idが数値でない）
        let raw_values = vec!["invalid".to_string(), "テスト".to_string()];

        let (row_data, errors) = reader_service.convert_raw_row(raw_values, &schema, 2);

        assert_eq!(row_data.row_number, 2);
        assert_eq!(row_data.values.len(), 2);
        assert_eq!(row_data.values[0], PostgresValue::Null); // エラー時はNULL
        assert_eq!(row_data.values[1], PostgresValue::Text("テスト".to_string()));
        assert_eq!(errors.len(), 1); // 1つのエラー
    }
}
