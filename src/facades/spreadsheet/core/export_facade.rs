/// スプレッドシートエクスポートFacade
///
/// PostgreSQLデータをGoogle Sheetsに書き込みます。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::env;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, QueryResult, Statement,
    TransactionTrait,
};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::errors::FacadeError;
use crate::infrastructure::database::repositories::SeaOrmGuildSpreadsheetConfigRepository;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, GoogleAuthServiceTrait,
    GuildSpreadsheetConfigService, GuildSpreadsheetConfigServiceTrait, PostgresType, PostgresValue,
    RegisteredTableSchema, SchemaExtractorService, SchemaExtractorServiceTrait,
    SpreadsheetReaderService, SpreadsheetReaderServiceTrait, SpreadsheetUrlService,
    SpreadsheetWriterService, SpreadsheetWriterServiceTrait, TableDefinition,
    TableDefinitionService, TableIO,
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

/// エクスポート設定
struct ExportConfig {
    /// エクスポート種別の名前（ログ用）
    export_type_name: &'static str,
    /// ギルドID（Noneの場合はグローバル）
    guild_id: Option<i64>,
}

impl ExportConfig {
    /// グローバル用設定
    fn global() -> Self {
        Self {
            export_type_name: "グローバルスプレッドシート",
            guild_id: None,
        }
    }

    /// ギルド用設定
    fn guild(guild_id: i64) -> Self {
        Self {
            export_type_name: "ギルド用スプレッドシート",
            guild_id: Some(guild_id),
        }
    }

    /// テーブルをフィルタするか判定
    fn should_include_table(&self, table: &RegisteredTableSchema) -> bool {
        match self.guild_id {
            // ギルド版: guild_master スキーマの登録テーブルのみ
            Some(_) => {
                crate::services::spreadsheet::get_schema_name(&table.table_name) == "guild_master"
            }
            // グローバル版: 全テーブル
            None => true,
        }
    }
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

    /// スプレッドシートにデータをエクスポート（内部共通処理）
    async fn export_spreadsheet_internal(
        &self,
        spreadsheet_id: &str,
        config: ExportConfig,
    ) -> Result<ExportResult, FacadeError> {
        info!("{}のエクスポートを開始します", config.export_type_name);

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

        // SeaORMエンティティからスキーマ定義を取得
        let schema_extractor = SchemaExtractorService::new();
        let registered_tables = schema_extractor.extract_registered_tables();

        // テーブルフィルタ（ギルド/グローバル）
        let target_tables: Vec<RegisteredTableSchema> = registered_tables
            .into_iter()
            .filter(|table| config.should_include_table(table))
            .collect();

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

        // RLS設定（ギルドの場合のみ）
        if let Some(guild_id) = config.guild_id {
            set_current_guild_id(&txn, guild_id).await?;
        }

        let result = async {
            let mut success_count = 0;
            let mut failure_count = 0;
            let mut total_rows = 0;
            let mut errors = Vec::new();

            for table_def in export_tables {
                let Some(table_schema) =
                    resolve_registered_table_schema(&target_tables, &table_def.table_name)
                else {
                    info!(
                        table_name = %table_def.table_name,
                        "{}のフィルタ条件により対象外のためスキップします",
                        config.export_type_name
                    );
                    continue;
                };

                let rows = match fetch_table_rows_from_database(
                    &txn,
                    &table_schema.table_name,
                    &table_schema.schema,
                )
                .await
                {
                    Ok(rows) => rows,
                    Err(e) => {
                        error!(
                            table_name = %table_def.table_name,
                            error = %e,
                            "データベースからのデータ取得に失敗しました"
                        );
                        failure_count += 1;
                        errors.push(format!("テーブル「{}」: {}", table_def.table_name, e));
                        continue;
                    }
                };

                info!(
                    table_name = %table_def.table_name,
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
                            table_name = %table_def.table_name,
                            rows_written = write_result.rows_written,
                            error_count = write_result.errors.len(),
                            "テーブルデータを書き込みました"
                        );

                        total_rows += write_result.rows_written;
                        success_count += 1;
                    }
                    Err(e) => {
                        error!(
                            table_name = %table_def.table_name,
                            error = %e,
                            "テーブルの書き込みに失敗しました"
                        );
                        failure_count += 1;
                        errors.push(format!("テーブル「{}」: {}", table_def.table_name, e));
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
                    "{}のエクスポートが完了しました",
                    config.export_type_name
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

    /// グローバルデータをスプレッドシートにエクスポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id))]
    pub async fn export_global_spreadsheet(
        &self,
        spreadsheet_id: &str,
    ) -> Result<ExportResult, FacadeError> {
        self.export_spreadsheet_internal(spreadsheet_id, ExportConfig::global())
            .await
    }

    /// ギルドデータをスプレッドシートにエクスポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id, guild_id = %guild_id))]
    pub async fn export_guild_spreadsheet(
        &self,
        spreadsheet_id: &str,
        guild_id: u64,
    ) -> Result<ExportResult, FacadeError> {
        self.export_spreadsheet_internal(spreadsheet_id, ExportConfig::guild(guild_id as i64))
            .await
    }

    /// ギルド設定（DB）からスプレッドシートIDを取得してエクスポートを実行
    /// - Facadeがトランザクション境界を管理
    /// - RLS用に `set_current_guild_id` を適用
    #[instrument(level = "info", skip(self), fields(guild_id = %guild_id))]
    pub async fn export_for_guild_by_config(
        &self,
        guild_id: i64,
    ) -> Result<ExportResult, FacadeError> {
        // まず、設定取得のためにTx開始しRLS設定
        let txn = self.db.begin().await?;
        set_current_guild_id(&txn, guild_id).await?;

        let config_service = GuildSpreadsheetConfigService::new(
            SeaOrmGuildSpreadsheetConfigRepository::new(),
            self.google_auth_service.clone(),
            SpreadsheetUrlService::new(),
        );
        let spreadsheet_id = match config_service
            .get_export_spreadsheet_id_with_txn(&txn, guild_id)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                txn.rollback().await?;
                return Err(FacadeError::BusinessRule {
                    source: crate::errors::BusinessRuleError::InvalidState {
                        entity: "GuildSpreadsheetConfig".to_string(),
                        current_state: "書き込み用スプレッドシート未登録".to_string(),
                    },
                });
            }
            Err(source) => {
                txn.rollback().await?;
                return Err(FacadeError::BusinessRule { source });
            }
        };

        // 設定取得成功 → commit
        txn.commit().await?;

        // 実際のエクスポート実行
        self.export_guild_spreadsheet(&spreadsheet_id, guild_id as u64)
            .await
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
                writeln!(f, "  - {error}")?;
            }
        }

        Ok(())
    }
}

fn resolve_registered_table_schema<'a>(
    registered_tables: &'a [RegisteredTableSchema],
    table_name: &str,
) -> Option<&'a RegisteredTableSchema> {
    registered_tables.iter().find(|table| {
        table.table_name == table_name || table.aliases.iter().any(|alias| alias == table_name)
    })
}

fn build_select_sql(table_name: &str, schema: &[ColumnSchema]) -> Result<String, FacadeError> {
    if schema.is_empty() {
        return Err(FacadeError::Database {
            source: DbErr::Custom(format!(
                "テーブル「{table_name}」のスキーマが空のため取得できません"
            )),
        });
    }

    let select_columns = schema
        .iter()
        .map(|column| format!("\"{}\"", column.column_name))
        .collect::<Vec<_>>()
        .join(", ");
    let schema_name = crate::services::spreadsheet::get_schema_name(table_name);

    Ok(format!(
        "SELECT {select_columns} FROM \"{schema_name}\".\"{table_name}\""
    ))
}

async fn fetch_table_rows_from_database(
    txn: &sea_orm::DatabaseTransaction,
    table_name: &str,
    schema: &[ColumnSchema],
) -> Result<Vec<Vec<PostgresValue>>, FacadeError> {
    let sql = build_select_sql(table_name, schema)?;
    let results = txn
        .query_all(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await?;

    results
        .iter()
        .map(|row| map_query_result_to_postgres_values(row, table_name, schema))
        .collect()
}

fn map_query_result_to_postgres_values(
    row: &QueryResult,
    table_name: &str,
    schema: &[ColumnSchema],
) -> Result<Vec<PostgresValue>, FacadeError> {
    schema
        .iter()
        .map(|column| map_column_value(row, table_name, column))
        .collect()
}

fn map_column_value(
    row: &QueryResult,
    table_name: &str,
    column: &ColumnSchema,
) -> Result<PostgresValue, FacadeError> {
    let column_name = column.column_name.as_str();
    match column.postgres_type {
        PostgresType::Integer => row
            .try_get::<Option<i32>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Integer))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::BigInt => row
            .try_get::<Option<i64>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::BigInt))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Text | PostgresType::Varchar => row
            .try_get::<Option<String>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Text))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Boolean => row
            .try_get::<Option<bool>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Boolean))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Timestamp => row
            .try_get::<Option<NaiveDateTime>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Timestamp))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::TimestampTz => row
            .try_get::<Option<DateTime<Utc>>>("", column_name)
            .map(|v| {
                v.map_or(PostgresValue::Null, |datetime| {
                    PostgresValue::TimestampTz(datetime.with_timezone(&Local))
                })
            })
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Date => row
            .try_get::<Option<NaiveDate>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Date))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Uuid => row
            .try_get::<Option<Uuid>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Uuid))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::Json | PostgresType::JsonB => row
            .try_get::<Option<serde_json::Value>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::Json))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::IntegerArray => row
            .try_get::<Option<Vec<i32>>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::IntegerArray))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
        PostgresType::TextArray => row
            .try_get::<Option<Vec<String>>>("", column_name)
            .map(|v| v.map_or(PostgresValue::Null, PostgresValue::TextArray))
            .map_err(|e| map_column_decode_error(table_name, column_name, e)),
    }
}

fn map_column_decode_error(table_name: &str, column_name: &str, error: DbErr) -> FacadeError {
    FacadeError::Database {
        source: DbErr::Custom(format!(
            "テーブル「{table_name}」のカラム「{column_name}」デコードに失敗しました: {error}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::spreadsheet::{ColumnSchema, PostgresType};

    #[test]
    fn resolve_registered_table_schema_matches_alias() {
        let tables = vec![RegisteredTableSchema {
            table_name: "message_texts".to_string(),
            aliases: vec!["messages".to_string()],
            schema: vec![ColumnSchema {
                column_name: "id".to_string(),
                postgres_type: PostgresType::Integer,
                nullable: false,
            }],
        }];

        let found = resolve_registered_table_schema(&tables, "messages");
        assert!(found.is_some());
        assert_eq!(found.unwrap().table_name, "message_texts");
    }

    #[test]
    fn build_select_sql_contains_schema_and_columns() {
        let schema = vec![
            ColumnSchema {
                column_name: "id".to_string(),
                postgres_type: PostgresType::Integer,
                nullable: false,
            },
            ColumnSchema {
                column_name: "display_name".to_string(),
                postgres_type: PostgresType::Text,
                nullable: false,
            },
        ];

        let sql = build_select_sql("battle_styles", &schema).expect("sql should be generated");
        assert!(sql.contains("\"master\".\"battle_styles\""));
        assert!(sql.contains("\"id\""));
        assert!(sql.contains("\"display_name\""));
    }
}
