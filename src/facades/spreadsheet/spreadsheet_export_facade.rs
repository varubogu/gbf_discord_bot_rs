/// スプレッドシートエクスポートFacade
///
/// PostgreSQLデータをGoogle Sheetsに書き込みます。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::collections::HashMap;
use std::env;

use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::{error, info, instrument, warn};

use crate::errors::{FacadeError, PresentationError};
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, GoogleAuthServiceTrait, PostgresValue,
    SpreadsheetReaderService, SpreadsheetReaderServiceTrait, SpreadsheetWriterService,
    SpreadsheetWriterServiceTrait, TableDefinition, TableDefinitionService, TableIO,
};

/// エクスポート結果
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// 成功したテーブル数
    pub success_count: usize,
    /// 失敗したテーブル数
    pub failure_count: usize,
    /// エクスポートした総行数
    pub total_rows: usize,
    /// エラーメッセージ
    pub errors: Vec<String>,
}

/// スプレッドシートエクスポートFacade
pub struct SpreadsheetExportFacade {
    db: DatabaseConnection,
    google_auth_service: GoogleAuthService,
}

impl SpreadsheetExportFacade {
    /// 新しいFacadeを作成
    pub fn new(db: DatabaseConnection) -> Result<Self, FacadeError> {
        // 環境変数からサービスアカウントキーファイルパスを取得
        let service_account_key_file =
            env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE").map_err(|_| {
                FacadeError::Initialization {
                    message: "環境変数 GOOGLE_SERVICE_ACCOUNT_KEY_FILE が設定されていません"
                        .to_string(),
                }
            })?;

        let google_auth_service = GoogleAuthService::new(service_account_key_file);

        Ok(Self {
            db,
            google_auth_service,
        })
    }

    /// グローバルデータをスプレッドシートにエクスポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id))]
    pub async fn export_global_spreadsheet(
        &self,
        spreadsheet_id: &str,
    ) -> Result<ExportResult, FacadeError> {
        info!("グローバルデータのエクスポートを開始します");

        // Google Sheets APIクライアントを取得
        let sheets_client = self.google_auth_service.get_sheets_client().await?;

        // TableDefinitionServiceとDataConverterServiceを作成
        let table_def_service = TableDefinitionService::new();
        let data_converter_service = DataConverterService::new();

        // SpreadsheetReaderServiceを作成（テーブル定義読み込み用）
        let reader_service = SpreadsheetReaderService::new(
            table_def_service.clone(),
            data_converter_service.clone(),
        );

        // SpreadsheetWriterServiceを作成
        let writer_service = SpreadsheetWriterService::new(data_converter_service);

        // テーブル定義を読み込み
        let table_definitions = reader_service
            .read_table_definitions(&sheets_client, spreadsheet_id)
            .await?;

        info!(
            table_count = table_definitions.len(),
            "テーブル定義を読み込みました"
        );

        // エクスポート対象のテーブルのみをフィルタ（table_io = Out or Both）
        let export_tables: Vec<TableDefinition> = table_definitions
            .into_iter()
            .filter(|def| def.table_io == TableIO::Out || def.table_io == TableIO::Both)
            .collect();

        if export_tables.is_empty() {
            warn!("エクスポート対象のテーブルが見つかりません");
            return Ok(ExportResult {
                success_count: 0,
                failure_count: 0,
                total_rows: 0,
                errors: vec!["エクスポート対象のテーブルが見つかりません".to_string()],
            });
        }

        info!(
            export_table_count = export_tables.len(),
            "エクスポート対象テーブルをフィルタしました"
        );

        // トランザクション開始（読み取り専用）
        let txn = self.db.begin().await?;

        let result = async {
            let mut success_count = 0;
            let mut failure_count = 0;
            let mut total_rows = 0;
            let mut errors = Vec::new();

            for table_def in export_tables {
                // TODO: データベースからデータを取得する処理
                // 現時点ではダミーデータ
                let rows: Vec<Vec<PostgresValue>> = Vec::new();

                info!(
                    table_name = %table_def.table_name_en,
                    row_count = rows.len(),
                    "データベースからデータを取得しました"
                );

                // データを書き込み
                match writer_service
                    .write_table_data(&sheets_client, spreadsheet_id, &table_def, rows.clone())
                    .await
                {
                    Ok(write_result) => {
                        if !write_result.errors.is_empty() {
                            for err in &write_result.errors {
                                errors.push(format!(
                                    "テーブル「{}」行{}: {}",
                                    err.table_name, err.row_number, err.message
                                ));
                            }
                        }

                        info!(
                            table_name = %table_def.table_name_en,
                            rows_written = write_result.rows_written,
                            error_count = write_result.errors.len(),
                            "テーブルデータを書き込みました"
                        );

                        total_rows += write_result.rows_written;
                        success_count += 1;
                    }
                    Err(e) => {
                        error!(
                            table_name = %table_def.table_name_en,
                            error = %e,
                            "テーブルの書き込みに失敗しました"
                        );
                        failure_count += 1;
                        errors.push(format!("テーブル「{}」: {}", table_def.table_name_en, e));
                    }
                }
            }

            Ok(ExportResult {
                success_count,
                failure_count,
                total_rows,
                errors,
            })
        }
        .await;

        match result {
            Ok(export_result) => {
                txn.commit().await?;
                info!(
                    success = export_result.success_count,
                    failure = export_result.failure_count,
                    total_rows = export_result.total_rows,
                    "グローバルデータのエクスポートが完了しました"
                );
                Ok(export_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "トランザクションをロールバックしました");
                Err(e)
            }
        }
    }

    /// ギルドデータをスプレッドシートにエクスポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id, guild_id = %guild_id))]
    pub async fn export_guild_spreadsheet(
        &self,
        spreadsheet_id: &str,
        guild_id: u64,
    ) -> Result<ExportResult, FacadeError> {
        info!(
            guild_id = %guild_id,
            "ギルドデータのエクスポートを開始します"
        );

        // グローバルと同様の処理（guild_idを考慮）
        // TODO: 実装を完成させる（現時点ではグローバルと同様）
        self.export_global_spreadsheet(spreadsheet_id).await
    }
}

impl std::fmt::Display for ExportResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "成功: {}テーブル, 失敗: {}テーブル, 総行数: {}行",
            self.success_count, self.failure_count, self.total_rows
        )?;

        if !self.errors.is_empty() {
            write!(f, "\n\n❌ エラー詳細:\n")?;
            for error in &self.errors {
                write!(f, "  - {}\n", error)?;
            }
        }

        Ok(())
    }
}
