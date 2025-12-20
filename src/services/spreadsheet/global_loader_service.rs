use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseBackend;
use sea_orm::sea_query::{
    Alias, ArrayType, Expr, IntoIden, PostgresQueryBuilder, Query, TableRef, Value as SeaValue,
};
use sea_orm::{ConnectionTrait, DatabaseTransaction, Statement};
use tracing::warn;

use crate::services::spreadsheet::google_auth_service::GoogleAuthServiceTrait;
use crate::services::spreadsheet::schema_extractor_service::SchemaExtractorServiceTrait;
use crate::services::spreadsheet::spreadsheet_reader_service::SpreadsheetReaderServiceTrait;
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, PostgresType, PostgresValue,
    SchemaExtractorService, SpreadsheetReaderService, TableDefinitionService,
};
use std::collections::HashMap;

use chrono::{DateTime, Utc};

/// テーブル名からスキーマ名を取得
///
/// テーブル名から適切なスキーマ名を返します。
fn get_schema_name(table_name: &str) -> &str {
    match table_name {
        // master スキーマ
        "quests"
        | "quest_aliases"
        | "battle_styles"
        | "elements"
        | "channel_types"
        | "event_schedules"
        | "event_schedule_details"
        | "message_texts"
        | "environments" => "master",
        // guild_master スキーマ
        "guilds" | "guild_channels" | "guild_spreadsheet_exports" | "guild_spreadsheet_imports" => {
            "guild_master"
        }
        // worker スキーマ
        "battle_recruitments"
        | "notifications"
        | "notification_rel_battle_recruitments"
        | "notification_rel_event_schedules"
        | "last_process_times" => "worker",
        // デフォルトはpublicスキーマ（後方互換性のため）
        _ => "public",
    }
}

/// テーブル名からスキーマ修飾されたTableRefを取得
///
/// スキーマ名とテーブル名を使用して、適切なTableRefを返します。
fn get_entity_table_ref(table_name: &str) -> TableRef {
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

/// グローバルスプレッドシート読み込み処理のService
///
/// 責務:
/// - グローバルスプレッドシート読み込みロジック
/// - データ変換処理
/// - データ検証処理
#[async_trait]
pub trait GlobalLoaderService: Send + Sync {
    /// グローバルスプレッドシートを開く
    async fn open_spreadsheet(&self) -> Result<()>;

    /// グローバルテーブルデータを読み込み
    async fn load_global_table_data(&self) -> Result<Vec<GlobalTableData>>;

    /// グローバルデータを変換
    async fn convert_global_data(
        &self,
        data: Vec<GlobalTableData>,
    ) -> Result<Vec<ConvertedGlobalData>>;

    /// グローバルデータを保存
    async fn save_global_data(
        &self,
        txn: &DatabaseTransaction,
        data: Vec<ConvertedGlobalData>,
    ) -> Result<()>;
}

/// グローバルテーブルデータ
#[derive(Debug, Clone)]
pub struct GlobalTableData {
    pub table_name: String,
    pub schema: Vec<ColumnSchema>,
    pub rows: Vec<Vec<PostgresValue>>,
    pub row_count: usize,
    pub errors: Vec<String>,
}

/// 変換済みグローバルデータ
#[derive(Debug, Clone)]
pub struct ConvertedGlobalData {
    pub table_name: String,
    pub schema: Vec<ColumnSchema>,
    pub rows: Vec<Vec<PostgresValue>>,
    pub row_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// グローバルスプレッドシート読み込みServiceの実装
pub struct GlobalLoaderServiceImpl {
    spreadsheet_id: String,
    google_auth_service: GoogleAuthService,
    table_definition_service: TableDefinitionService,
    data_converter_service: DataConverterService,
    reader_service: SpreadsheetReaderService<TableDefinitionService, DataConverterService>,
    schema_extractor: SchemaExtractorService,
}

impl GlobalLoaderServiceImpl {
    pub fn new(spreadsheet_id: String, service_account_key_file: String) -> Self {
        let table_definition_service = TableDefinitionService::new();
        let data_converter_service = DataConverterService::new();
        let reader_service = SpreadsheetReaderService::new(
            table_definition_service.clone(),
            data_converter_service.clone(),
        );
        let schema_extractor = SchemaExtractorService::new();

        Self {
            spreadsheet_id,
            google_auth_service: GoogleAuthService::new(service_account_key_file),
            table_definition_service,
            data_converter_service,
            reader_service,
            schema_extractor,
        }
    }
}

#[async_trait]
impl GlobalLoaderService for GlobalLoaderServiceImpl {
    async fn open_spreadsheet(&self) -> Result<()> {
        tracing::info!("グローバルスプレッドシート接続を開始");
        self.google_auth_service
            .get_sheets_client()
            .await
            .map_err(|e| crate::types::AppError::Generic(e.to_string()))?;
        tracing::info!("グローバルスプレッドシート接続完了");
        Ok(())
    }

    async fn load_global_table_data(&self) -> Result<Vec<GlobalTableData>> {
        tracing::info!("グローバルテーブルデータ読み込みを開始");

        let sheets_client = self
            .google_auth_service
            .get_sheets_client()
            .await
            .map_err(|e| crate::types::AppError::Generic(e.to_string()))?;

        let table_definitions = self
            .reader_service
            .read_table_definitions(&sheets_client, &self.spreadsheet_id)
            .await
            .map_err(|e| crate::types::AppError::Generic(e.to_string()))?;

        let registered_tables = self.schema_extractor.extract_registered_tables();

        let mut schema_map: HashMap<String, Vec<ColumnSchema>> = HashMap::new();
        for registered in registered_tables {
            schema_map.insert(registered.table_name.clone(), registered.schema.clone());
            for alias in registered.aliases {
                schema_map.insert(alias, registered.schema.clone());
            }
        }

        let mut result_tables = Vec::new();

        for table_def in table_definitions
            .into_iter()
            .filter(|def| def.table_io.can_import())
        {
            let Some(schema) = schema_map.get(&table_def.table_name) else {
                warn!(
                    table_name = %table_def.table_name,
                    "スキーマ情報が見つからないためスキップします"
                );
                continue;
            };

            let read_result = self
                .reader_service
                .read_table_data(&sheets_client, &self.spreadsheet_id, &table_def, schema)
                .await
                .map_err(|e| crate::types::AppError::Generic(e.to_string()))?;

            let mut rows = Vec::with_capacity(read_result.rows.len());
            for row in &read_result.rows {
                rows.push(row.values.clone());
            }

            let mut errors = Vec::new();
            for err in &read_result.errors {
                errors.push(format!("行{}: {}", err.row_number, err.message));
            }

            result_tables.push(GlobalTableData {
                table_name: table_def.table_name.clone(),
                schema: schema.clone(),
                rows,
                row_count: read_result.rows.len(),
                errors,
            });
        }

        tracing::info!(
            "グローバルテーブルデータ読み込み完了: {} テーブル",
            result_tables.len()
        );
        Ok(result_tables)
    }

    async fn convert_global_data(
        &self,
        data: Vec<GlobalTableData>,
    ) -> Result<Vec<ConvertedGlobalData>> {
        tracing::info!("グローバルデータ変換を開始: {} テーブル", data.len());

        let converted_data: Vec<ConvertedGlobalData> = data
            .into_iter()
            .map(|table_data| ConvertedGlobalData {
                table_name: table_data.table_name,
                schema: table_data.schema,
                rows: table_data.rows,
                row_count: table_data.row_count,
                errors: table_data.errors,
                warnings: Vec::new(),
            })
            .collect();

        tracing::info!(
            "グローバルデータ変換完了: {} テーブル",
            converted_data.len()
        );
        Ok(converted_data)
    }

    async fn save_global_data(
        &self,
        txn: &DatabaseTransaction,
        data: Vec<ConvertedGlobalData>,
    ) -> Result<()> {
        tracing::info!("グローバルデータ保存を開始: {} テーブル", data.len());

        for table in data {
            tracing::info!(
                table_name = %table.table_name,
                row_count = table.row_count,
                "テーブルデータの保存を開始します"
            );

            if !table.errors.is_empty() {
                for error in &table.errors {
                    warn!(
                        table_name = %table.table_name,
                        error = %error,
                        "変換時の警告を検出しました"
                    );
                }
            }

            if !table.warnings.is_empty() {
                for warning_msg in &table.warnings {
                    warn!(
                        table_name = %table.table_name,
                        warning = %warning_msg,
                        "変換時の追加警告"
                    );
                }
            }

            if table.schema.is_empty() {
                warn!(
                    table_name = %table.table_name,
                    "スキーマにカラム定義がないため、このテーブルをスキップします"
                );
                continue;
            }

            // 既存データを削除
            delete_table_rows(txn, &table.table_name).await?;

            if table.rows.is_empty() {
                tracing::info!(
                    table_name = %table.table_name,
                    "レコードが存在しないため削除のみ実施しました"
                );
                continue;
            }

            let mut insert_statement = build_insert_statement(&table.table_name, &table.schema);
            let mut inserted_rows = 0usize;

            for row in &table.rows {
                if row.len() != table.schema.len() {
                    warn!(
                        table_name = %table.table_name,
                        expected = table.schema.len(),
                        actual = row.len(),
                        "スキーマ列数とデータ列数が一致しないため、この行をスキップします"
                    );
                    continue;
                }

                let mut row_values = Vec::with_capacity(table.schema.len());
                for (value, column) in row.iter().zip(table.schema.iter()) {
                    let sea_value = postgres_value_to_sea_value(value.clone(), column);
                    row_values.push(sea_value);
                }

                let exprs: Vec<_> = row_values.into_iter().map(Expr::value).collect();

                if let Err(err) = insert_statement.values(exprs) {
                    return Err(crate::types::AppError::Generic(format!(
                        "テーブル「{}」のINSERT値生成に失敗しました: {}",
                        table.table_name, err
                    )));
                }

                inserted_rows += 1;
            }

            if inserted_rows == 0 {
                tracing::info!(
                    table_name = %table.table_name,
                    "挿入対象レコードがないためINSERTは実行しません"
                );
                continue;
            }

            let (sql, values) = insert_statement.build(PostgresQueryBuilder);
            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                sql,
                values,
            ))
            .await?;

            tracing::info!(
                table_name = %table.table_name,
                inserted_rows,
                "テーブルデータの保存が完了しました"
            );
        }

        tracing::info!("グローバルデータ保存完了");
        Ok(())
    }
}

async fn delete_table_rows(txn: &DatabaseTransaction, table_name: &str) -> Result<()> {
    let table_ref = get_entity_table_ref(table_name);
    let mut delete = Query::delete();
    delete.from_table(table_ref);
    let (sql, values) = delete.build(PostgresQueryBuilder);
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        values,
    ))
    .await?;
    Ok(())
}

fn build_insert_statement(
    table_name: &str,
    schema: &[ColumnSchema],
) -> sea_orm::sea_query::InsertStatement {
    let table_ref = get_entity_table_ref(table_name);
    let mut insert = Query::insert();
    insert
        .into_table(table_ref)
        .columns(schema.iter().map(|col| Alias::new(col.column_name.clone())));
    insert
}

fn postgres_value_to_sea_value(value: PostgresValue, column: &ColumnSchema) -> SeaValue {
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
        PostgresValue::Integer(v) => SeaValue::Int(Some(v)),
        PostgresValue::BigInt(v) => SeaValue::BigInt(Some(v)),
        PostgresValue::Text(v) => SeaValue::String(Some(Box::new(v))),
        PostgresValue::Boolean(v) => SeaValue::Bool(Some(v)),
        PostgresValue::Timestamp(v) => SeaValue::ChronoDateTime(Some(Box::new(v))),
        PostgresValue::TimestampTz(v) => {
            #[allow(deprecated)]
            let utc = DateTime::<Utc>::from_naive_utc_and_offset(v.naive_utc(), Utc);
            SeaValue::ChronoDateTimeUtc(Some(Box::new(utc)))
        }
        PostgresValue::Date(v) => SeaValue::ChronoDate(Some(Box::new(v))),
        PostgresValue::Uuid(v) => SeaValue::Uuid(Some(Box::new(v))),
        PostgresValue::Json(v) => SeaValue::Json(Some(Box::new(v))),
        PostgresValue::IntegerArray(v) => SeaValue::Array(
            ArrayType::Int,
            Some(Box::new(
                v.into_iter().map(|n| SeaValue::Int(Some(n))).collect(),
            )),
        ),
        PostgresValue::TextArray(v) => SeaValue::Array(
            ArrayType::String,
            Some(Box::new(
                v.into_iter()
                    .map(|s| SeaValue::String(Some(Box::new(s))))
                    .collect(),
            )),
        ),
    }
}
