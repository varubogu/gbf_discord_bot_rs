use std::collections::HashMap;

use sea_orm::TransactionTrait;
use tracing::{error, info, warn};

use crate::errors::FacadeError;
use crate::facades::scheduler::SchedulerFacade;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::spreadsheet::{
    DataConverterService, GeneratedUuidInfo, GoogleAuthServiceTrait, RegisteredTableSchema,
    SchemaExtractorService, SchemaExtractorServiceTrait, SpreadsheetReaderService,
    SpreadsheetReaderServiceTrait, SpreadsheetWriterService, SpreadsheetWriterServiceTrait,
    TableDefinition, TableDefinitionService, TableIO,
};
use crate::types::AUTO_RECRUITMENT_GLOBAL_RULE_GUILD_ID;

use super::table_processing::{
    process_single_table, take_table_definition, validate_imported_match_rule_data,
};
use super::{ImportConfig, ImportResult, SpreadsheetImportFacade, TableResultInfo, TableStatus};

impl SpreadsheetImportFacade {
    /// スプレッドシートからデータをインポート（内部共通処理）
    pub(super) async fn import_spreadsheet_internal(
        &self,
        spreadsheet_id: &str,
        config: ImportConfig,
    ) -> Result<ImportResult, FacadeError> {
        info!("{}のインポートを開始します", config.import_type_name);

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

        let (reader_service, target_tables, import_tables) =
            prepare_import_tables(&sheets_client, spreadsheet_id, &config).await?;

        if import_tables.is_empty() {
            warn!("インポート対象のテーブルが見つかりません");
            return Ok(ImportResult {
                success_count: 0,
                failure_count: 0,
                total_rows: 0,
                errors: vec!["インポート対象のテーブルが見つかりません".to_string()],
                warnings: Vec::new(),
                generated_uuids: Vec::new(),
                table_results: Vec::new(),
            });
        }

        info!(
            import_table_count = import_tables.len(),
            "インポート対象テーブルをフィルタしました"
        );

        // トランザクション開始
        let txn = self.db.begin().await?;

        // RLS設定（ギルドの場合のみ）
        if let Some(guild_id) = config.guild_id {
            set_current_guild_id(&txn, guild_id).await?;
        }

        let result = process_all_tables(
            &txn,
            &reader_service,
            &sheets_client,
            spreadsheet_id,
            &target_tables,
            import_tables,
            &config,
        )
        .await;

        match result {
            Ok(import_result) => {
                self.finalize_import(txn, &sheets_client, spreadsheet_id, import_result, &config)
                    .await
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "トランザクションをロールバックしました");
                Err(e)
            }
        }
    }

    /// UUID書き戻し・コミット・スケジュール再生成を行う
    async fn finalize_import(
        &self,
        txn: sea_orm::DatabaseTransaction,
        sheets_client: &google_sheets4::Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
        >,
        spreadsheet_id: &str,
        import_result: ImportResult,
        config: &ImportConfig,
    ) -> Result<ImportResult, FacadeError> {
        // 生成されたUUIDをスプレッドシートに書き戻し（コミット前に実施）
        if let Err(e) = write_back_uuids(
            sheets_client,
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
            return Err(e);
        }

        // UUID書き戻しが成功した場合のみコミット
        txn.commit().await?;
        info!(
            success = import_result.success_count,
            failure = import_result.failure_count,
            total_rows = import_result.total_rows,
            "{}のインポートが完了しました",
            config.import_type_name
        );

        // スケジュール再生成（global/guild共通、通知のみ）
        if let Some(task_type) = config.schedule_regeneration_task_type() {
            info!(
                guild_id = ?config.guild_id,
                task_type = task_type.as_i32(),
                "スケジュール自動再生成を開始します"
            );
            let scheduler_facade = SchedulerFacade::new(self.app_state.clone());
            let result = match config.guild_id {
                Some(guild_id) => {
                    scheduler_facade
                        .generate_schedules_for_guild(guild_id, Some(task_type))
                        .await
                }
                None => {
                    scheduler_facade
                        .generate_schedules_global(Some(task_type))
                        .await
                }
            };

            if let Err(e) = result {
                warn!(
                    error = %e,
                    "スケジュール自動再生成に失敗しました（インポート自体は成功）"
                );
            } else {
                info!("スケジュール自動再生成が完了しました");
            }
        }

        Ok(import_result)
    }
}

/// テーブル定義とスキーマフィルタを準備する
///
/// # 戻り値
/// `(SpreadsheetReaderService, フィルタ済み対象テーブル, インポート対象テーブル定義)`
async fn prepare_import_tables(
    sheets_client: &google_sheets4::Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
    spreadsheet_id: &str,
    config: &ImportConfig,
) -> Result<
    (
        SpreadsheetReaderService<TableDefinitionService, DataConverterService>,
        Vec<RegisteredTableSchema>,
        Vec<TableDefinition>,
    ),
    FacadeError,
> {
    // TableDefinitionServiceとDataConverterServiceを作成
    let table_def_service = TableDefinitionService::new();
    let data_converter_service = DataConverterService::new();

    // SpreadsheetReaderServiceを作成
    let reader_service = SpreadsheetReaderService::new(table_def_service, data_converter_service);

    // テーブル定義を読み込み
    let table_definitions = reader_service
        .read_table_definitions(sheets_client, spreadsheet_id)
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

    // インポート対象のテーブルのみをフィルタ（table_io = In or Both）
    let import_tables: Vec<TableDefinition> = table_definitions
        .into_iter()
        .filter(|def| def.table_io == TableIO::In || def.table_io == TableIO::Both)
        .collect();

    Ok((reader_service, target_tables, import_tables))
}

/// 対象テーブルを順に処理し、結果を集計する
#[allow(clippy::too_many_arguments)]
async fn process_all_tables(
    txn: &sea_orm::DatabaseTransaction,
    reader_service: &SpreadsheetReaderService<TableDefinitionService, DataConverterService>,
    sheets_client: &google_sheets4::Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
    spreadsheet_id: &str,
    target_tables: &[RegisteredTableSchema],
    import_tables: Vec<TableDefinition>,
    config: &ImportConfig,
) -> Result<ImportResult, FacadeError> {
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut total_rows = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut all_generated_uuids = Vec::new();
    let mut table_results = Vec::new();

    let mut import_table_map: HashMap<String, TableDefinition> = import_tables
        .into_iter()
        .map(|def| (def.table_name.clone(), def))
        .collect();

    for table in target_tables {
        let table_def = match take_table_definition(&mut import_table_map, table) {
            Some(def) => def,
            None => continue,
        };

        let result = process_single_table(
            txn,
            reader_service,
            sheets_client,
            spreadsheet_id,
            &table_def,
            &table.schema,
            config.guild_id,
        )
        .await;

        // 結果を集計
        errors.extend(result.errors);
        warnings.extend(result.warnings);
        all_generated_uuids.extend(result.generated_uuids);
        total_rows += result.inserted_rows;

        if result.success {
            success_count += 1;
            table_results.push(TableResultInfo {
                sheet_name: table_def.sheet_name.clone(),
                status: TableStatus::Success,
            });
        } else {
            failure_count += 1;
            table_results.push(TableResultInfo {
                sheet_name: table_def.sheet_name.clone(),
                status: TableStatus::Failed,
            });
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
        table_results.push(TableResultInfo {
            sheet_name: leftover.sheet_name,
            status: TableStatus::Skipped,
        });
    }

    if let Some(guild_id) = config.guild_id {
        validate_imported_match_rule_data(txn, guild_id).await?;
    } else {
        validate_imported_match_rule_data(txn, AUTO_RECRUITMENT_GLOBAL_RULE_GUILD_ID).await?;
    }

    Ok(ImportResult {
        success_count,
        failure_count,
        total_rows,
        errors,
        warnings,
        generated_uuids: all_generated_uuids,
        table_results,
    })
}

/// 生成されたUUIDをスプレッドシートに書き戻す
async fn write_back_uuids(
    sheets_client: &google_sheets4::Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
    >,
    spreadsheet_id: &str,
    generated_uuids: &[(String, GeneratedUuidInfo)],
) -> Result<(), FacadeError> {
    if generated_uuids.is_empty() {
        return Ok(());
    }

    info!(
        uuid_count = generated_uuids.len(),
        "生成されたUUIDをスプレッドシートに書き戻します"
    );

    // UUID情報のみを抽出
    let uuid_infos: Vec<_> = generated_uuids
        .iter()
        .map(|(_, info)| info.clone())
        .collect();

    // SpreadsheetWriterServiceを使用して書き戻し
    let data_converter = DataConverterService::new();
    let writer_service = SpreadsheetWriterService::new(data_converter);

    writer_service
        .write_back_generated_uuids(sheets_client, spreadsheet_id, &uuid_infos)
        .await
        .map_err(|e| {
            error!(error = %e, "UUID書き戻しに失敗しました");
            FacadeError::ExternalService { source: e }
        })?;

    info!("UUID書き戻しが完了しました");
    Ok(())
}
