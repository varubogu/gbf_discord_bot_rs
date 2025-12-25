/// スプレッドシートインポートFacade
///
/// Google Sheetsからデータを読み込み、PostgreSQLに保存します。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::{collections::HashMap, env};

use chrono::Utc;
use sea_orm::DbErr;
use sea_orm::sea_query::{
    Alias, ArrayType, Expr, PostgresQueryBuilder, Query, Value as SeaValue,
};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, Statement,
    TransactionTrait,
};
use tracing::{error, info, instrument, warn};

use crate::errors::FacadeError;
use crate::facades::scheduler::SchedulerFacade;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::{GuildSpreadsheetConfigRepository, GuildSpreadsheetConfigRepositoryTrait};
use crate::services::spreadsheet::{
    ColumnSchema, DataConverterService, GoogleAuthService, GoogleAuthServiceTrait, PostgresType,
    PostgresValue, RegisteredTableSchema, RowData, SchemaExtractorService,
    SchemaExtractorServiceTrait, SpreadsheetReaderService, SpreadsheetReaderServiceTrait,
    TableDefinition, TableDefinitionService, TableIO,
    get_entity_table_ref, get_schema_name,
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
                            None, // グローバル版ではguild_idなし
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
                        .write_back_generated_uuids(
                            &sheets_client,
                            spreadsheet_id,
                            &import_result.generated_uuids,
                        )
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
                                    "UUID書き戻しに失敗しました。次回読み込み時のID不整合を防ぐため、DB登録もロールバックしました: {e}"
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

    /// ギルド用スプレッドシートIDを取得
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    ///
    /// # 戻り値
    /// スプレッドシートID（未設定の場合はNone）
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    #[instrument(level = "info", skip(self), fields(guild_id = %guild_id))]
    pub async fn get_guild_spreadsheet_id(
        &self,
        guild_id: i64,
    ) -> Result<Option<String>, FacadeError> {
        info!(
            guild_id = guild_id,
            "ギルド用スプレッドシートID取得を開始します"
        );

        // トランザクション開始（Facade層の責務）
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| FacadeError::TransactionError {
                message: format!("トランザクション開始に失敗しました: {e}"),
            })?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id)
            .await
            .map_err(|e| FacadeError::TransactionError {
                message: format!("セッション変数設定に失敗しました: {e}"),
            })?;

        let result = async {
            let repository = GuildSpreadsheetConfigRepository::new();
            let spreadsheet_id = GuildSpreadsheetConfigRepositoryTrait::find_import_spreadsheet_id(
                &repository,
                &txn,
                guild_id,
            )
            .await?;

            Ok::<_, FacadeError>(spreadsheet_id)
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(spreadsheet_id) => {
                txn.commit()
                    .await
                    .map_err(|e| FacadeError::TransactionError {
                        message: format!("トランザクションコミットに失敗しました: {e}"),
                    })?;
                info!(
                    guild_id = guild_id,
                    spreadsheet_id = ?spreadsheet_id,
                    "ギルド用スプレッドシートID取得に成功しました"
                );
                Ok(spreadsheet_id)
            }
            Err(e) => {
                txn.rollback()
                    .await
                    .map_err(|e| FacadeError::TransactionError {
                        message: format!("トランザクションロールバックに失敗しました: {e}"),
                    })?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルド用スプレッドシートID取得に失敗しました"
                );
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

        // ギルド版のテーブルのみをフィルタ（guild_で始まるテーブル）
        let guild_tables: Vec<RegisteredTableSchema> = registered_tables
            .into_iter()
            .filter(|table| table.table_name.starts_with("guild_"))
            .collect();

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

        // RLSポリシーのためにセッション変数を設定
        use crate::infrastructure::database::db_helper::set_current_guild_id;
        set_current_guild_id(&txn, guild_id as i64).await?;

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

            for table in &guild_tables {
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
                            Some(guild_id as i64), // ギルド版ではguild_idを渡す
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
                        .write_back_generated_uuids(
                            &sheets_client,
                            spreadsheet_id,
                            &import_result.generated_uuids,
                        )
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
                                    "UUID書き戻しに失敗しました。次回読み込み時のID不整合を防ぐため、DB登録もロールバックしました: {e}"
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
                    "ギルド用スプレッドシートのインポートが完了しました"
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

    /// 生成されたUUIDをスプレッドシートに書き戻す
    #[instrument(level = "info", skip(self, sheets_client, generated_uuids))]
    async fn write_back_generated_uuids(
        &self,
        sheets_client: &google_sheets4::Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
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
                .or_default()
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
                let range = format!("'{sheet_name}'!{column_letter}{row_number}");

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
                    .map_err(|e| FacadeError::ExternalService {
                        source: crate::errors::ExternalServiceError::GoogleSheetsApiError {
                            message: format!("UUID書き戻しに失敗しました: {e}"),
                        },
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
                writeln!(f, "  - {warning}")?;
            }
        }

        if !self.errors.is_empty() {
            write!(f, "\n\n❌ エラー詳細:\n")?;
            for error in &self.errors {
                writeln!(f, "  - {error}")?;
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
        "battle_styles"  // battle_recruitments, quests, battle_recruitment_schedulesから参照
        | "quests"       // quest_aliases, battle_recruitments, battle_recruitment_schedulesから参照
        | "elements"     // recruitment_participantsから参照
        | "channel_types" // guild_channelsから参照
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
    // スプレッドシートに存在するIDのリストを作成（全テーブル共通）
    let id_list: Vec<i32> = inserted_ids
        .iter()
        .filter_map(|v| match v {
            PostgresValue::Integer(id) => Some(*id),
            _ => None,
        })
        .collect();

    match table_name {
        "battle_styles" => {
            // battle_recruitments, quests, battle_recruitment_schedulesから参照されているレコードは削除しない
            let schema_name = get_schema_name("battle_styles");
            let delete_sql = if id_list.is_empty() {
                format!(
                    "DELETE FROM {schema_name}.{table_name} WHERE id NOT IN (
                        SELECT DISTINCT battle_style_id FROM worker.battle_recruitments
                        UNION SELECT DISTINCT default_battle_style_id FROM master.quests
                        UNION SELECT DISTINCT battle_style_id FROM worker.battle_recruitment_schedules
                    )"
                )
            } else {
                let placeholders: Vec<String> =
                    (1..=id_list.len()).map(|i| format!("${i}")).collect();
                format!(
                    "DELETE FROM {}.{} WHERE id NOT IN ({}) AND id NOT IN (
                        SELECT DISTINCT battle_style_id FROM worker.battle_recruitments
                        UNION SELECT DISTINCT default_battle_style_id FROM master.quests
                        UNION SELECT DISTINCT battle_style_id FROM worker.battle_recruitment_schedules
                    )",
                    schema_name, table_name, placeholders.join(", ")
                )
            };

            if id_list.is_empty() {
                txn.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    delete_sql,
                ))
                .await
                .map_err(FacadeError::from)?;
            } else {
                let values: Vec<SeaValue> =
                    id_list.iter().map(|id| SeaValue::Int(Some(*id))).collect();
                txn.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    delete_sql,
                    values,
                ))
                .await
                .map_err(FacadeError::from)?;
            }
            tracing::debug!("battle_stylesテーブルから未参照レコードを削除しました");
        }
        "quests" => {
            // quest_aliases, battle_recruitments, battle_recruitment_schedulesから参照されているレコードは削除しない
            let schema_name = get_schema_name("quests");
            let delete_sql = if id_list.is_empty() {
                format!(
                    "DELETE FROM {schema_name}.{table_name} WHERE id NOT IN (
                        SELECT DISTINCT quest_id FROM master.quest_aliases
                        UNION SELECT DISTINCT quest_id FROM worker.battle_recruitments
                        UNION SELECT DISTINCT quest_id FROM worker.battle_recruitment_schedules
                    )"
                )
            } else {
                let placeholders: Vec<String> =
                    (1..=id_list.len()).map(|i| format!("${i}")).collect();
                format!(
                    "DELETE FROM {}.{} WHERE id NOT IN ({}) AND id NOT IN (
                        SELECT DISTINCT quest_id FROM master.quest_aliases
                        UNION SELECT DISTINCT quest_id FROM worker.battle_recruitments
                        UNION SELECT DISTINCT quest_id FROM worker.battle_recruitment_schedules
                    )",
                    schema_name,
                    table_name,
                    placeholders.join(", ")
                )
            };

            if id_list.is_empty() {
                txn.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    delete_sql,
                ))
                .await
                .map_err(FacadeError::from)?;
            } else {
                let values: Vec<SeaValue> =
                    id_list.iter().map(|id| SeaValue::Int(Some(*id))).collect();
                txn.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    delete_sql,
                    values,
                ))
                .await
                .map_err(FacadeError::from)?;
            }
            tracing::debug!("questsテーブルから未参照レコードを削除しました");
        }
        "elements" => {
            // recruitment_participantsから参照されているレコードは削除しない
            let schema_name = get_schema_name("elements");
            let delete_sql = if id_list.is_empty() {
                format!(
                    "DELETE FROM {schema_name}.{table_name} WHERE id NOT IN (
                        SELECT DISTINCT element_id FROM worker.recruitment_participants
                    )"
                )
            } else {
                let placeholders: Vec<String> =
                    (1..=id_list.len()).map(|i| format!("${i}")).collect();
                format!(
                    "DELETE FROM {}.{} WHERE id NOT IN ({}) AND id NOT IN (
                        SELECT DISTINCT element_id FROM worker.recruitment_participants
                    )",
                    schema_name,
                    table_name,
                    placeholders.join(", ")
                )
            };

            if id_list.is_empty() {
                txn.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    delete_sql,
                ))
                .await
                .map_err(FacadeError::from)?;
            } else {
                let values: Vec<SeaValue> =
                    id_list.iter().map(|id| SeaValue::Int(Some(*id))).collect();
                txn.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    delete_sql,
                    values,
                ))
                .await
                .map_err(FacadeError::from)?;
            }
            tracing::debug!("elementsテーブルから未参照レコードを削除しました");
        }
        "channel_types" => {
            // guild_channelsから参照されているレコードは削除しない
            let schema_name = get_schema_name("channel_types");
            let guild_schema = get_schema_name("guild_channels");
            let delete_sql = if id_list.is_empty() {
                format!(
                    "DELETE FROM {schema_name}.{table_name} WHERE id NOT IN (SELECT DISTINCT channel_type FROM {guild_schema}.guild_channels)"
                )
            } else {
                let placeholders: Vec<String> =
                    (1..=id_list.len()).map(|i| format!("${i}")).collect();
                format!(
                    "DELETE FROM {}.{} WHERE id NOT IN ({}) AND id NOT IN (SELECT DISTINCT channel_type FROM {}.guild_channels)",
                    schema_name,
                    table_name,
                    placeholders.join(", "),
                    guild_schema
                )
            };

            if id_list.is_empty() {
                txn.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    delete_sql,
                ))
                .await
                .map_err(FacadeError::from)?;
            } else {
                let values: Vec<SeaValue> =
                    id_list.iter().map(|id| SeaValue::Int(Some(*id))).collect();
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
    guild_id: Option<i64>,
) -> Result<(usize, Vec<String>), FacadeError> {
    let mut warnings = Vec::new();
    let table_ref = get_entity_table_ref(table_name);

    // UPSERT対象テーブル以外は全削除してから挿入
    if !should_use_upsert(table_name) {
        // スキーマ修飾されたDELETE文を生成
        let mut delete = Query::delete();
        delete.from_table(table_ref.clone());
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

    // guild_idが指定されている場合、INSERT時にguild_idカラムを追加
    let insert_columns: Vec<Alias> = if guild_id.is_some() {
        // guild_idカラムを先頭に追加
        let mut columns = vec![Alias::new("guild_id")];
        columns.extend(
            filtered_schema
                .iter()
                .map(|col| Alias::new(col.column_name.clone())),
        );
        columns
    } else {
        filtered_schema
            .iter()
            .map(|col| Alias::new(col.column_name.clone()))
            .collect()
    };

    let mut insert = Query::insert();
    insert
        .into_table(table_ref.clone())
        .columns(insert_columns.into_iter());

    let mut inserted_rows = 0usize;

    for row in rows {
        // スキーマには既にguild_id、created_at、updated_atが除外されているため、そのまま比較
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

        // created_atとupdated_atに対応する値を除外してINSERT値を作成
        let mut filtered_values: Vec<_> = Vec::new();

        // guild_idが指定されている場合、先頭にguild_idを追加
        if let Some(gid) = guild_id {
            filtered_values.push(Expr::value(SeaValue::BigInt(Some(gid))));
        }

        // スキーマの各列に対応する値を追加（created_atとupdated_atは既に除外されている）
        for (value, column) in row.values.iter().zip(schema.iter()) {
            if column.column_name != "created_at" && column.column_name != "updated_at" {
                filtered_values.push(Expr::value(postgres_value_to_sea_value(value, column)));
            }
        }

        insert
            .values(filtered_values)
            .map_err(|err| FacadeError::Database {
                source: DbErr::Custom(format!(
                    "テーブル「{table_name}」のINSERT値生成に失敗しました: {err}"
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
            // ローカルタイムゾーン（JST）をUTCに正しく変換
            let utc = v.with_timezone(&Utc);
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
