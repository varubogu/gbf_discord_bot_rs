use std::collections::HashMap;

use tracing::{error, info};

use crate::errors::FacadeError;
use crate::infrastructure::database::repositories::{
    SeaOrmQuestRepository,
    auto_recruitment::{
        SeaOrmAutoRecruitmentMatchRuleQuotaRepository, SeaOrmAutoRecruitmentMatchRuleRepository,
    },
};
use crate::services::auto_recruitment::match_rule_validation_service::AutoRecruitmentMatchRuleValidationService;
use crate::services::spreadsheet::{
    DataConverterService, GeneratedUuidInfo, RegisteredTableSchema, SpreadsheetPersistenceService,
    SpreadsheetReaderService, SpreadsheetReaderServiceTrait, TableDefinition,
    TableDefinitionService,
};

/// 単一テーブル処理結果
pub(super) struct TableProcessResult {
    pub(super) success: bool,
    pub(super) inserted_rows: usize,
    pub(super) errors: Vec<String>,
    pub(super) warnings: Vec<String>,
    pub(super) generated_uuids: Vec<(String, GeneratedUuidInfo)>,
}

/// 単一テーブルのインポート処理
pub(super) async fn process_single_table(
    txn: &sea_orm::DatabaseTransaction,
    reader_service: &SpreadsheetReaderService<TableDefinitionService, DataConverterService>,
    sheets_client: &google_sheets4::Sheets<
        google_sheets4::hyper_rustls::HttpsConnector<google_sheets4::hyper::client::HttpConnector>,
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
    persist_table_result(
        txn,
        table_def,
        table_schema,
        &read_result.rows,
        guild_id,
        errors,
        generated_uuids,
    )
    .await
}

/// 読み込んだ行データをDBへ永続化し、結果を`TableProcessResult`へまとめる
async fn persist_table_result(
    txn: &sea_orm::DatabaseTransaction,
    table_def: &TableDefinition,
    table_schema: &[crate::services::spreadsheet::ColumnSchema],
    rows: &[crate::services::spreadsheet::RowData],
    guild_id: Option<i64>,
    mut errors: Vec<String>,
    generated_uuids: Vec<(String, GeneratedUuidInfo)>,
) -> TableProcessResult {
    let persistence_service = SpreadsheetPersistenceService::new();
    match persistence_service
        .persist_table_data(txn, &table_def.table_name, table_schema, rows, guild_id)
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

/// 自動募集マッチングルール取り込み後の整合性検証
pub(super) async fn validate_imported_match_rule_data(
    txn: &sea_orm::DatabaseTransaction,
    scope_guild_id: i64,
) -> Result<(), FacadeError> {
    let validation_service = AutoRecruitmentMatchRuleValidationService::new(
        SeaOrmAutoRecruitmentMatchRuleRepository::new(),
        SeaOrmQuestRepository::new(),
        SeaOrmAutoRecruitmentMatchRuleQuotaRepository::new(),
    );

    validation_service
        .validate_guild_rules(txn, scope_guild_id)
        .await
        .map_err(|source| FacadeError::BusinessRule { source })?;

    Ok(())
}

/// テーブル定義マップから該当エントリを取り出す（本名・別名対応）
pub(super) fn take_table_definition(
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
