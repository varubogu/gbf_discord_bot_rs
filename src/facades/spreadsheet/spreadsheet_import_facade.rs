/// スプレッドシートインポートFacade
///
/// Google Sheetsからデータを読み込み、PostgreSQLに保存します。
/// トランザクション管理を行い、複数のServiceを協調させます。

use std::collections::HashMap;
use std::env;

use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::{error, info, instrument, warn};

use crate::errors::{FacadeError, PresentationError};
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, GoogleAuthServiceTrait,
    SpreadsheetReaderService, SpreadsheetReaderServiceTrait, TableDefinition,
    TableDefinitionService, TableIO,
};

/// インポート結果
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// 成功したテーブル数
    pub success_count: usize,
    /// 失敗したテーブル数
    pub failure_count: usize,
    /// インポートした総行数
    pub total_rows: usize,
    /// エラーメッセージ
    pub errors: Vec<String>,
}

/// スプレッドシートインポートFacade
pub struct SpreadsheetImportFacade {
    db: DatabaseConnection,
    google_auth_service: GoogleAuthService,
}

impl SpreadsheetImportFacade {
    /// 新しいFacadeを作成
    pub fn new(db: DatabaseConnection) -> Result<Self, FacadeError> {
        // 環境変数からサービスアカウントキーファイルパスを取得
        let service_account_key_file = env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE")
            .map_err(|_| FacadeError::Initialization {
                message: "環境変数 GOOGLE_SERVICE_ACCOUNT_KEY_FILE が設定されていません".to_string(),
            })?;

        let google_auth_service = GoogleAuthService::new(service_account_key_file);

        Ok(Self {
            db,
            google_auth_service,
        })
    }

    /// グローバルスプレッドシートからデータをインポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id))]
    pub async fn import_global_spreadsheet(
        &self,
        spreadsheet_id: &str,
    ) -> Result<ImportResult, FacadeError> {
        info!("グローバルスプレッドシートのインポートを開始します");

        // Google Sheets APIクライアントを取得
        let sheets_client = self
            .google_auth_service
            .get_sheets_client()
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    error_debug = ?e,
                    "Google Sheets APIクライアントの取得に失敗しました"
                );
                FacadeError::from(e)
            })?;

        // TableDefinitionServiceとDataConverterServiceを作成
        let table_def_service = TableDefinitionService::new();
        let data_converter_service = DataConverterService::new();

        // SpreadsheetReaderServiceを作成
        let reader_service =
            SpreadsheetReaderService::new(table_def_service, data_converter_service);

        // テーブル定義を読み込み
        let table_definitions = reader_service
            .read_table_definitions(&sheets_client, spreadsheet_id)
            .await?;

        info!(
            table_count = table_definitions.len(),
            "テーブル定義を読み込みました"
        );

        // TODO: 実際のスキーマ定義をモデルから取得する
        // 現時点では空のHashMapを使用（実装時に置き換え）
        let schemas: HashMap<String, Vec<ColumnSchema>> = HashMap::new();

        // インポート対象のテーブルのみをフィルタ（table_io = In or Both）
        let import_tables: Vec<TableDefinition> = table_definitions
            .into_iter()
            .filter(|def| def.table_io == TableIO::In || def.table_io == TableIO::Both)
            .collect();

        if import_tables.is_empty() {
            warn!("インポート対象のテーブルが見つかりません");
            return Ok(ImportResult {
                success_count: 0,
                failure_count: 0,
                total_rows: 0,
                errors: vec!["インポート対象のテーブルが見つかりません".to_string()],
            });
        }

        info!(
            import_table_count = import_tables.len(),
            "インポート対象テーブルをフィルタしました"
        );

        // トランザクション開始
        let txn = self.db.begin().await?;

        let result = async {
            let mut success_count = 0;
            let mut failure_count = 0;
            let mut total_rows = 0;
            let mut errors = Vec::new();

            for table_def in import_tables {
                // スキーマを取得（TODO: 実際の実装）
                let schema = match schemas.get(&table_def.table_name_en) {
                    Some(s) => s,
                    None => {
                        warn!(
                            table_name = %table_def.table_name_en,
                            "スキーマが見つかりません。スキップします"
                        );
                        failure_count += 1;
                        errors.push(format!(
                            "テーブル「{}」: スキーマが見つかりません",
                            table_def.table_name_en
                        ));
                        continue;
                    }
                };

                // データを読み込み
                match reader_service
                    .read_table_data(&sheets_client, spreadsheet_id, &table_def, schema)
                    .await
                {
                    Ok(read_result) => {
                        if !read_result.errors.is_empty() {
                            for err in &read_result.errors {
                                errors.push(format!(
                                    "テーブル「{}」行{}: {}",
                                    err.table_name, err.row_number, err.message
                                ));
                            }
                        }

                        // TODO: データベースに保存する処理
                        // 現時点ではログ出力のみ
                        info!(
                            table_name = %table_def.table_name_en,
                            row_count = read_result.rows.len(),
                            error_count = read_result.errors.len(),
                            "テーブルデータを読み込みました"
                        );

                        total_rows += read_result.rows.len();
                        success_count += 1;
                    }
                    Err(e) => {
                        error!(
                            table_name = %table_def.table_name_en,
                            error = %e,
                            "テーブルの読み込みに失敗しました"
                        );
                        failure_count += 1;
                        errors.push(format!(
                            "テーブル「{}」: {}",
                            table_def.table_name_en, e
                        ));
                    }
                }
            }

            Ok(ImportResult {
                success_count,
                failure_count,
                total_rows,
                errors,
            })
        }
        .await;

        match result {
            Ok(import_result) => {
                txn.commit().await?;
                info!(
                    success = import_result.success_count,
                    failure = import_result.failure_count,
                    total_rows = import_result.total_rows,
                    "グローバルスプレッドシートのインポートが完了しました"
                );
                Ok(import_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "トランザクションをロールバックしました");
                Err(e)
            }
        }
    }

    /// ギルド用スプレッドシートからデータをインポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id, guild_id = %guild_id))]
    pub async fn import_guild_spreadsheet(
        &self,
        spreadsheet_id: &str,
        guild_id: u64,
    ) -> Result<ImportResult, FacadeError> {
        info!(
            guild_id = %guild_id,
            "ギルド用スプレッドシートのインポートを開始します"
        );

        // グローバルと同様の処理（guild_idを考慮）
        // TODO: 実装を完成させる（現時点ではグローバルと同様）
        self.import_global_spreadsheet(spreadsheet_id).await
    }
}

impl std::fmt::Display for ImportResult {
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
