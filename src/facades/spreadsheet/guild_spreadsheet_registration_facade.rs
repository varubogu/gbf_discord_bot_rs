/// GuildSpreadsheetRegistrationFacade
///
/// ギルドスプレッドシート登録のユースケースを実現
/// トランザクション管理と複数サービスの協調を担当
use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::{error, info, instrument};

use crate::errors::FacadeError;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::GuildSpreadsheetConfigRepository;
use crate::services::spreadsheet::{
    GoogleAuthService, GuildSpreadsheetConfigService,
    GuildSpreadsheetConfigServiceTrait, SpreadsheetUrlService, SpreadsheetUrlServiceTrait,
};

/// 登録結果
#[derive(Debug, Clone)]
pub struct RegistrationResult {
    /// 読み込み用スプレッドシートURL
    pub load_spreadsheet_url: String,
    /// 書き込み用スプレッドシートURL
    pub push_spreadsheet_url: String,
}

/// GuildSpreadsheetRegistrationFacade
pub struct GuildSpreadsheetRegistrationFacade {
    db: DatabaseConnection,
    google_auth_service: GoogleAuthService,
    url_service: SpreadsheetUrlService,
}

impl GuildSpreadsheetRegistrationFacade {
    /// 新しいFacadeを作成
    pub fn new(db: DatabaseConnection) -> Result<Self, FacadeError> {
        // 環境変数からサービスアカウントキーファイルパスを取得
        let service_account_key_file =
            std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE").map_err(|_| {
                FacadeError::Initialization {
                    message: "環境変数 GOOGLE_SERVICE_ACCOUNT_KEY_FILE が設定されていません"
                        .to_string(),
                }
            })?;

        let google_auth_service = GoogleAuthService::new(service_account_key_file);
        let url_service = SpreadsheetUrlService::new();

        Ok(Self {
            db,
            google_auth_service,
            url_service,
        })
    }

    /// ギルドスプレッドシートを登録
    #[instrument(level = "info", skip(self), fields(guild_id = %guild_id))]
    pub async fn register_guild_spreadsheets(
        &self,
        guild_id: i64,
        load_spreadsheet_url: &str,
        push_spreadsheet_url: &str,
    ) -> Result<RegistrationResult, FacadeError> {
        info!("ギルドスプレッドシート登録を開始します");

        // URL正規化とID抽出
        let load_spreadsheet_id = self
            .url_service
            .extract_spreadsheet_id(load_spreadsheet_url)?;
        let push_spreadsheet_id = self
            .url_service
            .extract_spreadsheet_id(push_spreadsheet_url)?;

        info!(
            load_id = %load_spreadsheet_id,
            push_id = %push_spreadsheet_id,
            "スプレッドシートIDを抽出しました"
        );

        // Repositoryを作成
        let repository = GuildSpreadsheetConfigRepository::new();

        // Serviceを作成
        let config_service = GuildSpreadsheetConfigService::new(
            self.db.clone(),
            repository.clone(),
            self.google_auth_service.clone(),
            self.url_service.clone(),
        );

        // 読み込み用スプレッドシートへのアクセス確認
        info!("読み込み用スプレッドシートのアクセス確認中...");
        config_service
            .verify_spreadsheet_access(&load_spreadsheet_id)
            .await
            .map_err(|e| {
                error!(error = %e, "読み込み用スプレッドシートへのアクセスに失敗しました");
                FacadeError::ExternalService { source: e }
            })?;

        // 書き込み用スプレッドシートへのアクセス確認
        info!("書き込み用スプレッドシートのアクセス確認中...");
        config_service
            .verify_spreadsheet_access(&push_spreadsheet_id)
            .await
            .map_err(|e| {
                error!(error = %e, "書き込み用スプレッドシートへのアクセスに失敗しました");
                FacadeError::ExternalService { source: e }
            })?;

        info!("スプレッドシートへのアクセス確認が完了しました");

        // トランザクション開始
        let txn = self.db.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        // トランザクション内で登録処理を実行
        let result = async {
            config_service
                .register_spreadsheets(&txn, guild_id, &load_spreadsheet_id, &push_spreadsheet_id)
                .await
                .map_err(|e| {
                    error!(error = %e, "スプレッドシート登録に失敗しました");
                    FacadeError::BusinessRule { source: e }
                })?;

            Ok::<_, FacadeError>(())
        }
        .await;

        // 結果に応じてコミット or ロールバック
        match result {
            Ok(_) => {
                txn.commit().await?;
                info!("スプレッドシート登録をコミットしました");

                // 正規化されたURLを生成
                let load_url = self.url_service.build_spreadsheet_url(&load_spreadsheet_id);
                let push_url = self.url_service.build_spreadsheet_url(&push_spreadsheet_id);

                Ok(RegistrationResult {
                    load_spreadsheet_url: load_url,
                    push_spreadsheet_url: push_url,
                })
            }
            Err(e) => {
                txn.rollback().await?;
                error!("スプレッドシート登録をロールバックしました");
                Err(e)
            }
        }
    }
}
