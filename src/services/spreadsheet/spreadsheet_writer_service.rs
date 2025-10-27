/// スプレッドシート書き込みサービス
///
/// PostgreSQLデータをGoogle Sheetsに書き込みます。
/// 設計書: docs/develop/design/spreadsheet/service_layer.md
use async_trait::async_trait;
use google_sheets4::{
    Sheets,
    api::{ClearValuesRequest, UpdateValuesResponse, ValueRange},
    hyper::client::HttpConnector,
    hyper_rustls::HttpsConnector,
};

use crate::errors::{ExternalServiceError, ValidationError};
use crate::services::spreadsheet::{DataConverterServiceTrait, PostgresValue, TableDefinition};

/// スプレッドシート書き込み結果
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// テーブル名（英語）
    pub table_name: String,
    /// 書き込んだ行数
    pub rows_written: usize,
    /// 書き込み時に発生したエラー
    pub errors: Vec<WriteError>,
}

/// 書き込みエラー
#[derive(Debug, Clone)]
pub struct WriteError {
    /// テーブル名
    pub table_name: String,
    /// 行番号
    pub row_number: usize,
    /// エラーメッセージ
    pub message: String,
}

/// スプレッドシート書き込みサービストレイト
#[async_trait]
pub trait SpreadsheetWriterServiceTrait: Send + Sync {
    /// テーブルデータをスプレッドシートに書き込み
    async fn write_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
        rows: Vec<Vec<PostgresValue>>,
    ) -> Result<WriteResult, ExternalServiceError>;

    /// シートをクリア（データのみ、ヘッダーは残す）
    async fn clear_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
    ) -> Result<(), ExternalServiceError>;

    /// 複数テーブルを一括書き込み
    async fn write_all_tables(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_data: Vec<(TableDefinition, Vec<Vec<PostgresValue>>)>,
    ) -> Result<Vec<WriteResult>, ExternalServiceError>;
}

/// スプレッドシート書き込みサービス実装
pub struct SpreadsheetWriterService<D>
where
    D: DataConverterServiceTrait,
{
    data_converter_service: D,
}

impl<D> SpreadsheetWriterService<D>
where
    D: DataConverterServiceTrait,
{
    pub fn new(data_converter_service: D) -> Self {
        Self {
            data_converter_service,
        }
    }

    /// シート名からデータ範囲を構築（A2以降、ヘッダー除く）
    fn build_data_range(sheet_name: &str) -> String {
        format!("'{}'!A2:ZZ", sheet_name)
    }

    /// PostgreSQL値の行をスプレッドシート文字列に変換
    fn convert_rows(
        &self,
        rows: Vec<Vec<PostgresValue>>,
        table_name: &str,
    ) -> Result<Vec<Vec<String>>, Vec<WriteError>> {
        let mut converted_rows = Vec::new();
        let mut errors = Vec::new();

        for (index, row) in rows.into_iter().enumerate() {
            let row_number = index + 2; // スプレッドシート上の行番号（ヘッダーが1行目）

            match self.data_converter_service.convert_row_to_spreadsheet(row) {
                Ok(string_row) => {
                    converted_rows.push(string_row);
                }
                Err(e) => {
                    errors.push(WriteError {
                        table_name: table_name.to_string(),
                        row_number,
                        message: e.to_string(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(converted_rows)
        } else {
            Err(errors)
        }
    }
}

#[async_trait]
impl<D> SpreadsheetWriterServiceTrait for SpreadsheetWriterService<D>
where
    D: DataConverterServiceTrait + Send + Sync,
{
    async fn write_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
        rows: Vec<Vec<PostgresValue>>,
    ) -> Result<WriteResult, ExternalServiceError> {
        let table_name = &table_definition.table_name_en;
        let sheet_name = &table_definition.table_name_jp;

        tracing::info!(
            table_name = %table_name,
            sheet_name = %sheet_name,
            row_count = rows.len(),
            "テーブルデータの書き込みを開始します"
        );

        // PostgreSQL値をスプレッドシート文字列に変換
        let string_rows = match self.convert_rows(rows, table_name) {
            Ok(rows) => rows,
            Err(errors) => {
                tracing::error!(
                    table_name = %table_name,
                    error_count = errors.len(),
                    "データ変換に失敗しました"
                );
                return Ok(WriteResult {
                    table_name: table_name.clone(),
                    rows_written: 0,
                    errors,
                });
            }
        };

        if string_rows.is_empty() {
            tracing::info!(
                table_name = %table_name,
                "書き込むデータがありません"
            );
            return Ok(WriteResult {
                table_name: table_name.clone(),
                rows_written: 0,
                errors: Vec::new(),
            });
        }

        // ValueRangeを作成（serde_json::Valueに変換）
        let json_values: Vec<Vec<serde_json::Value>> = string_rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| serde_json::Value::String(cell.clone()))
                    .collect()
            })
            .collect();

        let value_range = ValueRange {
            major_dimension: Some("ROWS".to_string()),
            range: Some(Self::build_data_range(sheet_name)),
            values: Some(json_values),
        };

        // データをクリア（既存データを削除）
        self.clear_table_data(sheets_client, spreadsheet_id, table_definition)
            .await?;

        // Google Sheets APIでデータを書き込み
        let result = sheets_client
            .spreadsheets()
            .values_update(
                value_range,
                spreadsheet_id,
                &Self::build_data_range(sheet_name),
            )
            .value_input_option("RAW")
            .doit()
            .await
            .map_err(|e| ExternalServiceError::GoogleSheetsApiError {
                message: format!("シート「{}」への書き込みに失敗しました: {}", sheet_name, e),
            })?;

        let rows_written = string_rows.len();

        tracing::info!(
            table_name = %table_name,
            rows_written = rows_written,
            "テーブルデータの書き込みが完了しました"
        );

        Ok(WriteResult {
            table_name: table_name.clone(),
            rows_written,
            errors: Vec::new(),
        })
    }

    async fn clear_table_data(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_definition: &TableDefinition,
    ) -> Result<(), ExternalServiceError> {
        let sheet_name = &table_definition.table_name_jp;
        let range = Self::build_data_range(sheet_name);

        tracing::debug!(
            table_name = %table_definition.table_name_en,
            sheet_name = %sheet_name,
            range = %range,
            "シートデータをクリアします"
        );

        let clear_request = ClearValuesRequest::default();

        sheets_client
            .spreadsheets()
            .values_clear(clear_request, spreadsheet_id, &range)
            .doit()
            .await
            .map_err(|e| ExternalServiceError::GoogleSheetsApiError {
                message: format!("シート「{}」のクリアに失敗しました: {}", sheet_name, e),
            })?;

        Ok(())
    }

    async fn write_all_tables(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
        table_data: Vec<(TableDefinition, Vec<Vec<PostgresValue>>)>,
    ) -> Result<Vec<WriteResult>, ExternalServiceError> {
        tracing::info!(
            table_count = table_data.len(),
            "全テーブルの書き込みを開始します"
        );

        let mut results = Vec::new();

        for (table_def, rows) in table_data {
            match self
                .write_table_data(sheets_client, spreadsheet_id, &table_def, rows)
                .await
            {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    tracing::error!(
                        table_name = %table_def.table_name_en,
                        error = %e,
                        "テーブルの書き込みに失敗しました"
                    );
                    // エラーがあっても他のテーブルは書き込み続行
                    results.push(WriteResult {
                        table_name: table_def.table_name_en.clone(),
                        rows_written: 0,
                        errors: vec![WriteError {
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
            "全テーブルの書き込みが完了しました"
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::spreadsheet::{DataConverterService, PostgresValue};

    #[test]
    fn test_build_data_range() {
        let range = SpreadsheetWriterService::<DataConverterService>::build_data_range("クエスト");
        assert_eq!(range, "'クエスト'!A2:ZZ");
    }

    #[tokio::test]
    async fn test_convert_rows() {
        let converter_service = DataConverterService::new();
        let writer_service = SpreadsheetWriterService::new(converter_service);

        let rows = vec![
            vec![
                PostgresValue::Integer(1),
                PostgresValue::Text("テスト1".to_string()),
            ],
            vec![
                PostgresValue::Integer(2),
                PostgresValue::Text("テスト2".to_string()),
            ],
        ];

        let result = writer_service.convert_rows(rows, "test_table");

        assert!(result.is_ok());
        let converted = result.unwrap();
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0], vec!["1", "テスト1"]);
        assert_eq!(converted[1], vec!["2", "テスト2"]);
    }

    #[tokio::test]
    async fn test_convert_rows_with_null() {
        let converter_service = DataConverterService::new();
        let writer_service = SpreadsheetWriterService::new(converter_service);

        let rows = vec![vec![
            PostgresValue::Integer(1),
            PostgresValue::Null,
            PostgresValue::Text("テスト".to_string()),
        ]];

        let result = writer_service.convert_rows(rows, "test_table");

        assert!(result.is_ok());
        let converted = result.unwrap();
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0], vec!["1", "", "テスト"]);
    }
}
