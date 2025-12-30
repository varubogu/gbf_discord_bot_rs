/// スプレッドシートインポートFacade
///
/// Google Sheetsからデータを読み込み、PostgreSQLに保存します。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::{collections::HashMap, env};

use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::{error, info, instrument, warn};

use crate::errors::FacadeError;
use crate::facades::scheduler::SchedulerFacade;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::{GuildSpreadsheetConfigRepository, GuildSpreadsheetConfigRepositoryTrait};
use crate::services::spreadsheet::{
    DataConverterService, GeneratedUuidInfo, GoogleAuthService, GoogleAuthServiceTrait,
    RegisteredTableSchema, SchemaExtractorService, SchemaExtractorServiceTrait,
    SpreadsheetPersistenceService, SpreadsheetReaderService, SpreadsheetReaderServiceTrait,
    SpreadsheetWriterService, SpreadsheetWriterServiceTrait, TableDefinition,
    TableDefinitionService, TableIO,
};
use crate::types::AppState;

/// テーブル処理ステータス
#[derive(Debug, Clone)]
pub enum TableStatus {
    /// 成功
    Success,
    /// 失敗
    Failed,
    /// 対象外（Botで扱わないテーブル）
    Skipped,
}

/// テーブル処理結果情報
#[derive(Debug, Clone)]
pub struct TableResultInfo {
    /// シート名
    pub sheet_name: String,
    /// 処理ステータス
    pub status: TableStatus,
}

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
    pub generated_uuids: Vec<(String, GeneratedUuidInfo)>, // (table_name, uuid_info)
    /// テーブル処理結果一覧（シート名と処理ステータス）
    pub table_results: Vec<TableResultInfo>,
}

/// スプレッドシートインポートFacade
pub struct SpreadsheetImportFacade {
    db: DatabaseConnection,
    google_auth_service: GoogleAuthService,
    app_state: std::sync::Arc<AppState>,
}

/// 単一テーブル処理結果
struct TableProcessResult {
    success: bool,
    inserted_rows: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    generated_uuids: Vec<(String, GeneratedUuidInfo)>,
}

/// インポート設定
struct ImportConfig {
    /// インポート種別の名前（ログ用）
    import_type_name: &'static str,
    /// ギルドID（Noneの場合はグローバル）
    guild_id: Option<i64>,
    /// スケジュール再生成を実行するか
    regenerate_schedule: bool,
}

impl ImportConfig {
    /// グローバル用設定
    fn global() -> Self {
        Self {
            import_type_name: "グローバルスプレッドシート",
            guild_id: None,
            regenerate_schedule: true,
        }
    }

    /// ギルド用設定
    fn guild(guild_id: i64) -> Self {
        Self {
            import_type_name: "ギルド用スプレッドシート",
            guild_id: Some(guild_id),
            regenerate_schedule: false,
        }
    }

    /// テーブルをフィルタするか判定
    fn should_include_table(&self, table: &RegisteredTableSchema) -> bool {
        match self.guild_id {
            // ギルド版: guild_で始まるテーブルのみ
            Some(_) => table.table_name.starts_with("guild_"),
            // グローバル版: 全テーブル
            None => true,
        }
    }
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

    /// 生成されたUUIDをスプレッドシートに書き戻す
    async fn write_back_uuids(
        sheets_client: &google_sheets4::Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
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

    /// 単一テーブルのインポート処理
    async fn process_single_table(
        txn: &sea_orm::DatabaseTransaction,
        reader_service: &SpreadsheetReaderService<TableDefinitionService, DataConverterService>,
        sheets_client: &google_sheets4::Sheets<
            google_sheets4::hyper_rustls::HttpsConnector<
                google_sheets4::hyper::client::HttpConnector,
            >,
        >,
        spreadsheet_id: &str,
        table_def: &TableDefinition,
        table_schema: &[crate::services::spreadsheet::ColumnSchema],
        guild_id: Option<i64>,
    ) -> TableProcessResult {
        // データ読み込み
        let read_result = match reader_service
            .read_table_data(sheets_client, spreadsheet_id, table_def, table_schema)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                error!(
                    table_name = %table_def.table_name,
                    error = %e,
                    "テーブルの読み込みに失敗しました"
                );
                return TableProcessResult {
                    success: false,
                    inserted_rows: 0,
                    errors: vec![format!("テーブル「{}」: {}", table_def.table_name, e)],
                    warnings: Vec::new(),
                    generated_uuids: Vec::new(),
                };
            }
        };

        // 読み込みエラーを収集
        let mut errors = Vec::new();
        for err in &read_result.errors {
            errors.push(format!(
                "テーブル「{}」行{}: {}",
                err.table_name, err.row_number, err.message
            ));
        }

        // 生成されたUUIDを記録
        let generated_uuids: Vec<_> = read_result
            .generated_uuids
            .iter()
            .map(|generated_uuid| {
                (
                    table_def.table_name.clone(),
                    GeneratedUuidInfo {
                        sheet_name: table_def.sheet_name.clone(),
                        row_number: generated_uuid.row_number,
                        column_index: generated_uuid.column_index,
                        uuid: generated_uuid.uuid,
                    },
                )
            })
            .collect();

        info!(
            table_name = %table_def.table_name,
            row_count = read_result.rows.len(),
            error_count = read_result.errors.len(),
            generated_uuid_count = read_result.generated_uuids.len(),
            "テーブルデータを読み込みました"
        );

        // データ永続化
        let persistence_service = SpreadsheetPersistenceService::new();
        match persistence_service
            .persist_table_data(
                txn,
                &table_def.table_name,
                table_schema,
                &read_result.rows,
                guild_id,
            )
            .await
        {
            Ok(persist_result) => TableProcessResult {
                success: true,
                inserted_rows: persist_result.inserted_rows,
                errors,
                warnings: persist_result.warnings,
                generated_uuids,
            },
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
                errors.push(message);
                TableProcessResult {
                    success: false,
                    inserted_rows: 0,
                    errors,
                    warnings: Vec::new(),
                    generated_uuids,
                }
            }
            Err(other) => {
                error!(
                    table_name = %table_def.table_name,
                    error = %other,
                    "テーブルデータの保存に失敗しました"
                );
                errors.push(format!("テーブル『{}』: {}", table_def.table_name, other));
                TableProcessResult {
                    success: false,
                    inserted_rows: 0,
                    errors,
                    warnings: Vec::new(),
                    generated_uuids,
                }
            }
        }
    }

    /// スプレッドシートからデータをインポート（内部共通処理）
    async fn import_spreadsheet_internal(
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

        let result = async {
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

            for table in &target_tables {
                let table_def = match take_table_definition(&mut import_table_map, table) {
                    Some(def) => def,
                    None => continue,
                };

                let result = Self::process_single_table(
                    &txn,
                    &reader_service,
                    &sheets_client,
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
        .await;

        match result {
            Ok(import_result) => {
                // 生成されたUUIDをスプレッドシートに書き戻し（コミット前に実施）
                if let Err(e) = Self::write_back_uuids(
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

                // スケジュール再生成（グローバルのみ）
                if config.regenerate_schedule {
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

    /// グローバルスプレッドシートからデータをインポート
    #[instrument(level = "info", skip(self), fields(spreadsheet_id = %spreadsheet_id))]
    pub async fn import_global_spreadsheet(
        &self,
        spreadsheet_id: &str,
    ) -> Result<ImportResult, FacadeError> {
        self.import_spreadsheet_internal(spreadsheet_id, ImportConfig::global())
            .await
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
        self.import_spreadsheet_internal(spreadsheet_id, ImportConfig::guild(guild_id as i64))
            .await
    }
}

impl std::fmt::Display for ImportResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "成功: {}テーブル, 失敗: {}テーブル, 総行数: {}行",
            self.success_count, self.failure_count, self.total_rows
        )?;

        // テーブル処理結果を1行ずつ表示
        if !self.table_results.is_empty() {
            write!(f, "\n")?;
            for table_result in &self.table_results {
                let (status_icon, status_text) = match table_result.status {
                    TableStatus::Success => ("✅", "成功"),
                    TableStatus::Failed => ("❌", "失敗"),
                    TableStatus::Skipped => ("⚠️", "対象外"),
                };
                writeln!(
                    f,
                    "{}{}: {}",
                    status_icon, table_result.sheet_name, status_text
                )?;
            }
        }

        if !self.warnings.is_empty() {
            write!(f, "\n⚠️ 警告:\n")?;
            for warning in &self.warnings {
                writeln!(f, "  - {warning}")?;
            }
        }

        if !self.errors.is_empty() {
            write!(f, "\n❌ エラー詳細:\n")?;
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
