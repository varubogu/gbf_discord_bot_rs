/// スプレッドシートインポートFacade
///
/// Google Sheetsからデータを読み込み、PostgreSQLに保存します。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::{collections::HashMap, env};

use chrono::{DateTime, Utc};
use sea_orm::DbErr;
use sea_orm::sea_query::{Alias, ArrayType, Expr, PostgresQueryBuilder, Query, Value as SeaValue};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, Statement,
    TransactionTrait,
};
use tracing::{error, info, instrument, warn};

use crate::errors::FacadeError;
use crate::facades::scheduler::SchedulerFacade;
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, GoogleAuthServiceTrait, PostgresType,
    PostgresValue, RegisteredTableSchema, RowData, SchemaExtractorService,
    SchemaExtractorServiceTrait, SpreadsheetReaderService, SpreadsheetReaderServiceTrait,
    TableDefinition, TableDefinitionService, TableIO,
};
use crate::types::AppState;

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
    /// 警告メッセージ
    pub warnings: Vec<String>,
}

/// スプレッドシートインポートFacade
pub struct SpreadsheetImportFacade {
    db: DatabaseConnection,
    google_auth_service: GoogleAuthService,
    app_state: std::sync::Arc<AppState>,
}

impl SpreadsheetImportFacade {
    /// 新しいFacadeを作成
    pub fn new(
        db: DatabaseConnection,
        app_state: std::sync::Arc<AppState>,
    ) -> Result<Self, FacadeError> {
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
            app_state,
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

        // SeaORMエンティティからスキーマ定義を取得
        let schema_extractor = SchemaExtractorService::new();
        let registered_tables = schema_extractor.extract_registered_tables();

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
                warnings: Vec::new(),
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
            let mut warnings = Vec::new();

            let mut import_table_map: HashMap<String, TableDefinition> = import_tables
                .into_iter()
                .map(|def| (def.table_name.clone(), def))
                .collect();

            for table in &registered_tables {
                let table_def = match take_table_definition(&mut import_table_map, table) {
                    Some(def) => def,
                    None => continue,
                };

                match reader_service
                    .read_table_data(&sheets_client, spreadsheet_id, &table_def, &table.schema)
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

                        info!(
                            table_name = %table_def.table_name,
                            row_count = read_result.rows.len(),
                            error_count = read_result.errors.len(),
                            "テーブルデータを読み込みました"
                        );

                        match persist_table_data(
                            &txn,
                            &table_def.table_name,
                            &table.schema,
                            &read_result.rows,
                        )
                        .await
                        {
                            Ok((inserted_rows, persist_warnings)) => {
                                total_rows += inserted_rows;
                                if !persist_warnings.is_empty() {
                                    warnings.extend(persist_warnings);
                                }
                                success_count += 1;
                            }
                            Err(FacadeError::Database { source }) => {
                                let message = format!(
                                    "テーブル『{}』: DB書き込みに失敗しました: {}",
                                    table_def.table_name, source
                                );
                                error!(
                                    table_name = %table_def.table_name,
                                    db_error = %source,
                                    "テーブルデータの保存に失敗しました"
                                );
                                failure_count += 1;
                                errors.push(message);
                            }
                            Err(other) => {
                                error!(
                                    table_name = %table_def.table_name,
                                    error = %other,
                                    "テーブルデータの保存に失敗しました"
                                );
                                failure_count += 1;
                                errors.push(format!(
                                    "テーブル『{}』: {}",
                                    table_def.table_name, other
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            table_name = %table_def.table_name,
                            error = %e,
                            "テーブルの読み込みに失敗しました"
                        );
                        failure_count += 1;
                        errors.push(format!("テーブル「{}」: {}", table_def.table_name, e));
                    }
                }
            }

            for leftover in import_table_map.into_values() {
                warn!(
                    table_name = %leftover.table_name,
                    sheet_name = %leftover.sheet_name,
                    "このBotでは未対応のテーブルのためスキップしました"
                );
                warnings.push(format!(
                    "⚠️ テーブル「{}」（シート: {}）はBotで扱わないため無視しました",
                    leftover.table_name, leftover.sheet_name
                ));
            }

            Ok(ImportResult {
                success_count,
                failure_count,
                total_rows,
                errors,
                warnings,
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

                // インポート成功後、スケジュールを自動再生成
                info!("スケジュール自動再生成を開始します");
                let scheduler_facade = SchedulerFacade::new(self.app_state.clone());
                if let Err(e) = scheduler_facade.generate_schedules().await {
                    warn!(
                        error = %e,
                        "スケジュール自動再生成に失敗しました（インポート自体は成功）"
                    );
                } else {
                    info!("スケジュール自動再生成が完了しました");
                }

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

        if !self.warnings.is_empty() {
            write!(f, "\n\n⚠️ 警告:\n")?;
            for warning in &self.warnings {
                write!(f, "  - {}\n", warning)?;
            }
        }

        if !self.errors.is_empty() {
            write!(f, "\n\n❌ エラー詳細:\n")?;
            for error in &self.errors {
                write!(f, "  - {}\n", error)?;
            }
        }

        Ok(())
    }
}

/// テーブル定義マップから該当エントリを取り出す（本名・別名対応）
fn take_table_definition(
    definitions: &mut HashMap<String, TableDefinition>,
    table: &RegisteredTableSchema,
) -> Option<TableDefinition> {
    if let Some(definition) = definitions.remove(&table.table_name) {
        return Some(definition);
    }

    for alias in &table.aliases {
        if let Some(definition) = definitions.remove(alias) {
            return Some(definition);
        }
    }

    None
}

async fn persist_table_data(
    txn: &DatabaseTransaction,
    table_name: &str,
    schema: &[ColumnSchema],
    rows: &[RowData],
) -> Result<(usize, Vec<String>), FacadeError> {
    let mut warnings = Vec::new();

    let mut delete = Query::delete();
    delete.from_table(Alias::new(table_name));
    let (delete_sql, delete_values) = delete.build(PostgresQueryBuilder);
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        delete_sql,
        delete_values,
    ))
    .await
    .map_err(FacadeError::from)?;

    if rows.is_empty() {
        return Ok((0, warnings));
    }

    // created_atとupdated_atを除外したスキーマを作成
    let filtered_schema: Vec<_> = schema
        .iter()
        .filter(|col| col.column_name != "created_at" && col.column_name != "updated_at")
        .collect();

    let mut insert = Query::insert();
    insert
        .into_table(Alias::new(table_name))
        .columns(filtered_schema.iter().map(|col| Alias::new(col.column_name.clone())));

    let mut inserted_rows = 0usize;

    for row in rows {
        if row.values.len() != schema.len() {
            warnings.push(format!(
                "テーブル「{}」行{}: 列数が一致しないためスキップしました (期待{}列/実際{}列)",
                table_name,
                row.row_number,
                schema.len(),
                row.values.len()
            ));
            continue;
        }

        // created_atとupdated_atに対応する値を除外
        let filtered_values: Vec<_> = row
            .values
            .iter()
            .zip(schema.iter())
            .filter(|(_, column)| column.column_name != "created_at" && column.column_name != "updated_at")
            .map(|(value, column)| Expr::value(postgres_value_to_sea_value(value, column)))
            .collect();

        insert.values(filtered_values).map_err(|err| FacadeError::Database {
            source: DbErr::Custom(format!(
                "テーブル「{}」のINSERT値生成に失敗しました: {}",
                table_name, err
            )),
        })?;

        inserted_rows += 1;
    }

    if inserted_rows == 0 {
        return Ok((0, warnings));
    }

    let (insert_sql, insert_values) = insert.build(PostgresQueryBuilder);
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        insert_sql,
        insert_values,
    ))
    .await
    .map_err(FacadeError::from)?;

    Ok((inserted_rows, warnings))
}

fn postgres_value_to_sea_value(value: &PostgresValue, column: &ColumnSchema) -> SeaValue {
    match value {
        PostgresValue::Null => match column.postgres_type {
            PostgresType::Integer => SeaValue::Int(None),
            PostgresType::BigInt => SeaValue::BigInt(None),
            PostgresType::Boolean => SeaValue::Bool(None),
            PostgresType::Timestamp => SeaValue::ChronoDateTime(None),
            PostgresType::TimestampTz => SeaValue::ChronoDateTimeUtc(None),
            PostgresType::Date => SeaValue::ChronoDate(None),
            PostgresType::Uuid => SeaValue::Uuid(None),
            PostgresType::Json | PostgresType::JsonB => SeaValue::Json(None),
            PostgresType::IntegerArray => SeaValue::Array(ArrayType::Int, None),
            PostgresType::TextArray => SeaValue::Array(ArrayType::String, None),
            _ => SeaValue::String(None),
        },
        PostgresValue::Integer(v) => SeaValue::Int(Some(*v)),
        PostgresValue::BigInt(v) => SeaValue::BigInt(Some(*v)),
        PostgresValue::Text(v) => SeaValue::String(Some(Box::new(v.clone()))),
        PostgresValue::Boolean(v) => SeaValue::Bool(Some(*v)),
        PostgresValue::Timestamp(v) => SeaValue::ChronoDateTime(Some(Box::new(*v))),
        PostgresValue::TimestampTz(v) => {
            #[allow(deprecated)]
            let utc = DateTime::<Utc>::from_naive_utc_and_offset(v.naive_utc(), Utc);
            SeaValue::ChronoDateTimeUtc(Some(Box::new(utc)))
        }
        PostgresValue::Date(v) => SeaValue::ChronoDate(Some(Box::new(*v))),
        PostgresValue::Uuid(v) => SeaValue::Uuid(Some(Box::new(*v))),
        PostgresValue::Json(v) => SeaValue::Json(Some(Box::new(v.clone()))),
        PostgresValue::IntegerArray(v) => SeaValue::Array(
            ArrayType::Int,
            Some(Box::new(
                v.iter().map(|n| SeaValue::Int(Some(*n))).collect(),
            )),
        ),
        PostgresValue::TextArray(v) => SeaValue::Array(
            ArrayType::String,
            Some(Box::new(
                v.iter()
                    .map(|s| SeaValue::String(Some(Box::new(s.clone()))))
                    .collect(),
            )),
        ),
    }
}
