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
    SpreadsheetWriterService, SpreadsheetWriterServiceTrait, TableDefinition,
    TableDefinitionService, TableIO,
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
    /// 自動生成されたUUID情報（テーブル名、シート名、行番号、列番号、UUID）
    pub generated_uuids: Vec<GeneratedUuidInfo>,
}

/// 自動生成されたUUID情報
#[derive(Debug, Clone)]
pub struct GeneratedUuidInfo {
    /// テーブル名
    pub table_name: String,
    /// シート名
    pub sheet_name: String,
    /// スプレッドシート上の行番号（1始まり）
    pub row_number: usize,
    /// スプレッドシート上の列番号（0始まり）
    pub column_index: usize,
    /// 生成されたUUID
    pub uuid: uuid::Uuid,
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
                generated_uuids: Vec::new(),
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
            let mut all_generated_uuids = Vec::new();

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

                        // 生成されたUUIDを記録
                        for generated_uuid in &read_result.generated_uuids {
                            all_generated_uuids.push(GeneratedUuidInfo {
                                table_name: table_def.table_name.clone(),
                                sheet_name: table_def.sheet_name.clone(),
                                row_number: generated_uuid.row_number,
                                column_index: generated_uuid.column_index,
                                uuid: generated_uuid.uuid,
                            });
                        }

                        info!(
                            table_name = %table_def.table_name,
                            row_count = read_result.rows.len(),
                            error_count = read_result.errors.len(),
                            generated_uuid_count = read_result.generated_uuids.len(),
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
                generated_uuids: all_generated_uuids,
            })
        }
        .await;

        match result {
            Ok(import_result) => {
                // 生成されたUUIDをスプレッドシートに書き戻し（コミット前に実施）
                if !import_result.generated_uuids.is_empty() {
                    info!(
                        uuid_count = import_result.generated_uuids.len(),
                        "生成されたUUIDをスプレッドシートに書き戻します"
                    );

                    if let Err(e) = self
                        .write_back_generated_uuids(&sheets_client, spreadsheet_id, &import_result.generated_uuids)
                        .await
                    {
                        error!(
                            error = %e,
                            "UUID書き戻しに失敗したため、トランザクションをロールバックします"
                        );
                        txn.rollback().await?;
                        return Err(FacadeError::ExternalService {
                            source: crate::errors::ExternalServiceError::GoogleSheetsApiError {
                                message: format!(
                                    "UUID書き戻しに失敗しました。次回読み込み時のID不整合を防ぐため、DB登録もロールバックしました: {}",
                                    e
                                ),
                            },
                        });
                    } else {
                        info!("UUID書き戻しが完了しました");
                    }
                }

                // UUID書き戻しが成功した場合のみコミット
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

    /// 生成されたUUIDをスプレッドシートに書き戻す
    #[instrument(level = "info", skip(self, sheets_client, generated_uuids))]
    async fn write_back_generated_uuids(
        &self,
        sheets_client: &google_sheets4::Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
        >,
        spreadsheet_id: &str,
        generated_uuids: &[GeneratedUuidInfo],
    ) -> Result<(), FacadeError> {
        use google_sheets4::api::ValueRange;

        // シート名ごとにグループ化
        let mut updates_by_sheet: HashMap<String, Vec<(usize, usize, String)>> = HashMap::new();

        for uuid_info in generated_uuids {
            updates_by_sheet
                .entry(uuid_info.sheet_name.clone())
                .or_insert_with(Vec::new)
                .push((
                    uuid_info.row_number,
                    uuid_info.column_index,
                    uuid_info.uuid.to_string(),
                ));
        }

        // シートごとに書き込み
        for (sheet_name, updates) in updates_by_sheet {
            for (row_number, column_index, uuid_str) in updates {
                // A1記法に変換（列番号をアルファベットに変換）
                let column_letter = column_index_to_letter(column_index);
                let range = format!("'{}'!{}{}", sheet_name, column_letter, row_number);

                let value_range = ValueRange {
                    values: Some(vec![vec![serde_json::Value::String(uuid_str.clone())]]),
                    range: Some(range.clone()),
                    ..Default::default()
                };

                sheets_client
                    .spreadsheets()
                    .values_update(value_range, spreadsheet_id, &range)
                    .value_input_option("USER_ENTERED")
                    .doit()
                    .await
                    .map_err(|e| {
                        FacadeError::ExternalService {
                            source: crate::errors::ExternalServiceError::GoogleSheetsApiError {
                                message: format!("UUID書き戻しに失敗しました: {}", e),
                            },
                        }
                    })?;

                tracing::debug!(
                    sheet_name = %sheet_name,
                    range = %range,
                    uuid = %uuid_str,
                    "UUIDを書き戻しました"
                );
            }
        }

        Ok(())
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

/// 列番号（0始まり）をA1記法の列文字列に変換
/// 例: 0 -> "A", 1 -> "B", 25 -> "Z", 26 -> "AA"
fn column_index_to_letter(index: usize) -> String {
    let mut result = String::new();
    let mut n = index + 1; // A1記法は1始まり

    while n > 0 {
        n -= 1; // 0ベースに変換
        let remainder = n % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        n /= 26;
    }

    result
}

/// テーブルがUPSERT方式で保存すべきかを判定
///
/// 参照マスタテーブルで、他のテーブルから外部キー参照されている場合は
/// DELETE + INSERTではなくUPSERTを使用する必要があります。
fn should_use_upsert(table_name: &str) -> bool {
    matches!(
        table_name,
        "channel_types" // guild_channelsから参照されているため
    )
}

/// UPSERT用のSQLクエリを構築
///
/// INSERT文にON CONFLICT句を追加して、既存レコードがあれば更新、なければ挿入します。
fn build_upsert_query(
    table_name: &str,
    insert_sql: &str,
    filtered_schema: &[&ColumnSchema],
) -> String {
    // テーブルごとのプライマリキーを定義
    let primary_key = match table_name {
        "channel_types" => "id",
        _ => "id", // デフォルトはid
    };

    // ON CONFLICT (primary_key) DO UPDATE SET column1 = EXCLUDED.column1, column2 = EXCLUDED.column2, ...
    let update_columns: Vec<String> = filtered_schema
        .iter()
        .filter(|col| col.column_name != primary_key) // プライマリキーは更新しない
        .map(|col| format!("{} = EXCLUDED.{}", col.column_name, col.column_name))
        .collect();

    format!(
        "{} ON CONFLICT ({}) DO UPDATE SET {}",
        insert_sql,
        primary_key,
        update_columns.join(", ")
    )
}

/// スプレッドシートに存在しないレコードを削除
///
/// 他のテーブルから参照されていないレコードのみ削除します。
/// 参照されているレコードは削除せず、警告として記録します。
async fn delete_unreferenced_records(
    txn: &DatabaseTransaction,
    table_name: &str,
    inserted_ids: &[&PostgresValue],
) -> Result<(), FacadeError> {
    match table_name {
        "channel_types" => {
            // スプレッドシートに存在するIDのリストを作成
            let id_list: Vec<i32> = inserted_ids
                .iter()
                .filter_map(|v| match v {
                    PostgresValue::Integer(id) => Some(*id),
                    _ => None,
                })
                .collect();

            if id_list.is_empty() {
                // 全削除（参照されていないもののみ）
                let delete_sql =
                    "DELETE FROM channel_types WHERE id NOT IN (SELECT DISTINCT channel_type FROM guild_channels)";
                txn.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    delete_sql,
                ))
                .await
                .map_err(FacadeError::from)?;
            } else {
                // スプレッドシートに存在しないIDを削除（参照されていないもののみ）
                let placeholders: Vec<String> = (1..=id_list.len())
                    .map(|i| format!("${}", i))
                    .collect();

                let delete_sql = format!(
                    "DELETE FROM channel_types WHERE id NOT IN ({}) AND id NOT IN (SELECT DISTINCT channel_type FROM guild_channels)",
                    placeholders.join(", ")
                );

                let values: Vec<SeaValue> = id_list
                    .iter()
                    .map(|id| SeaValue::Int(Some(*id)))
                    .collect();

                txn.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    delete_sql,
                    values,
                ))
                .await
                .map_err(FacadeError::from)?;
            }

            tracing::debug!("channel_typesテーブルから未参照レコードを削除しました");
        }
        _ => {
            // その他のテーブルは何もしない
        }
    }
    Ok(())
}

async fn persist_table_data(
    txn: &DatabaseTransaction,
    table_name: &str,
    schema: &[ColumnSchema],
    rows: &[RowData],
) -> Result<(usize, Vec<String>), FacadeError> {
    let mut warnings = Vec::new();

    // UPSERT対象テーブル以外は全削除してから挿入
    if !should_use_upsert(table_name) {
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
    }

    if rows.is_empty() {
        // UPSERT対象テーブルの場合、空でも全削除が必要
        if should_use_upsert(table_name) {
            delete_unreferenced_records(txn, table_name, &[]).await?;
        }
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

    // UPSERT対象テーブルの場合はON CONFLICT句を追加
    if should_use_upsert(table_name) {
        let (insert_sql, insert_values) = insert.build(PostgresQueryBuilder);

        // ON CONFLICT句を手動で追加（SeaORMのQueryBuilderはUPSERTをサポートしていないため）
        let upsert_sql = build_upsert_query(table_name, &insert_sql, &filtered_schema);

        txn.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            upsert_sql,
            insert_values,
        ))
        .await
        .map_err(FacadeError::from)?;

        // スプレッドシートに存在しないレコードを削除（参照されていないもののみ）
        let inserted_ids: Vec<&PostgresValue> = rows
            .iter()
            .filter_map(|row| {
                // プライマリキー列（通常は最初の列）の値を取得
                row.values.first()
            })
            .collect();

        delete_unreferenced_records(txn, table_name, &inserted_ids).await?;
    } else {
        let (insert_sql, insert_values) = insert.build(PostgresQueryBuilder);
        txn.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            insert_sql,
            insert_values,
        ))
        .await
        .map_err(FacadeError::from)?;
    }

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
