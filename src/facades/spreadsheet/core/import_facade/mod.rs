/// スプレッドシートインポートFacade
///
/// Google Sheetsからデータを読み込み、PostgreSQLに保存します。
/// トランザクション管理を行い、複数のServiceを協調させます。
use std::env;

use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::{error, info, instrument};

use crate::errors::FacadeError;
use crate::infrastructure::database::repositories::SeaOrmGuildSpreadsheetConfigRepository;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::services::spreadsheet::{
    GeneratedUuidInfo, GoogleAuthService, GuildSpreadsheetConfigService,
    GuildSpreadsheetConfigServiceTrait, RegisteredTableSchema, SpreadsheetUrlService,
};
use crate::types::AppState;

mod orchestration;
mod table_processing;

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

/// インポート設定
struct ImportConfig {
    /// インポート種別の名前（ログ用）
    import_type_name: &'static str,
    /// ギルドID（Noneの場合はグローバル）
    guild_id: Option<i64>,
    /// 自動再生成対象のタスク種別（Noneの場合は再生成しない）
    schedule_regeneration_task_type: Option<ScheduledTaskType>,
}

impl ImportConfig {
    /// グローバル用設定
    fn global() -> Self {
        Self {
            import_type_name: "グローバルスプレッドシート",
            guild_id: None,
            schedule_regeneration_task_type: Some(ScheduledTaskType::Notification),
        }
    }

    /// ギルド用設定
    fn guild(guild_id: i64) -> Self {
        Self {
            import_type_name: "ギルド用スプレッドシート",
            guild_id: Some(guild_id),
            schedule_regeneration_task_type: Some(ScheduledTaskType::Notification),
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

    fn schedule_regeneration_task_type(&self) -> Option<ScheduledTaskType> {
        self.schedule_regeneration_task_type
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
            let config_service = GuildSpreadsheetConfigService::new(
                SeaOrmGuildSpreadsheetConfigRepository::new(),
                self.google_auth_service.clone(),
                SpreadsheetUrlService::new(),
            );
            let spreadsheet_id = config_service
                .get_import_spreadsheet_id_with_txn(&txn, guild_id)
                .await
                .map_err(|source| FacadeError::BusinessRule { source })?;

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
            writeln!(f)?;
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

#[cfg(test)]
mod tests {
    use super::{ImportConfig, ScheduledTaskType};

    #[test]
    fn global_import_config_regenerates_notification_for_all_guilds() {
        let config = ImportConfig::global();

        assert_eq!(config.guild_id, None);
        assert_eq!(
            config.schedule_regeneration_task_type(),
            Some(ScheduledTaskType::Notification)
        );
    }

    #[test]
    fn guild_import_config_regenerates_notification_for_target_guild() {
        let config = ImportConfig::guild(12345);

        assert_eq!(config.guild_id, Some(12345));
        assert_eq!(
            config.schedule_regeneration_task_type(),
            Some(ScheduledTaskType::Notification)
        );
    }
}
