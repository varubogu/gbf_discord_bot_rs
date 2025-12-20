/// GuildSpreadsheetConfigService
///
/// ギルドスプレッドシート設定に関するビジネスロジック
/// - スプレッドシートへのアクセス権限確認
/// - Repository層への委譲
use crate::errors::{BusinessRuleError, ExternalServiceError};
use crate::repository::GuildSpreadsheetConfigRepositoryTrait;
use crate::services::spreadsheet::{GoogleAuthServiceTrait, SpreadsheetUrlServiceTrait};
use async_trait::async_trait;
use google_sheets4::Sheets;
use google_sheets4::hyper::client::HttpConnector;
use google_sheets4::hyper_rustls::HttpsConnector;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

#[async_trait]
pub trait GuildSpreadsheetConfigServiceTrait: Send + Sync {
    /// スプレッドシートへのアクセス権限を確認
    async fn verify_spreadsheet_access(
        &self,
        spreadsheet_id: &str,
    ) -> Result<(), ExternalServiceError>;

    /// 読み込み用・書き込み用スプレッドシートを登録
    async fn register_spreadsheets(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        load_spreadsheet_id: &str,
        push_spreadsheet_id: &str,
    ) -> Result<(), BusinessRuleError>;

    /// 読み込み用スプレッドシートIDを取得
    async fn get_import_spreadsheet_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<String>, BusinessRuleError>;

    /// 書き込み用スプレッドシートIDを取得
    async fn get_export_spreadsheet_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<String>, BusinessRuleError>;
}

pub struct GuildSpreadsheetConfigService<R, G, U>
where
    R: GuildSpreadsheetConfigRepositoryTrait,
    G: GoogleAuthServiceTrait,
    U: SpreadsheetUrlServiceTrait,
{
    repository: R,
    google_auth_service: G,
    #[allow(unused)]
    url_service: U,
}

impl<R, G, U> GuildSpreadsheetConfigService<R, G, U>
where
    R: GuildSpreadsheetConfigRepositoryTrait,
    G: GoogleAuthServiceTrait,
    U: SpreadsheetUrlServiceTrait,
{
    pub fn new(repository: R, google_auth_service: G, url_service: U) -> Self {
        Self {
            repository,
            google_auth_service,
            url_service,
        }
    }

    /// Google Sheets APIを使ってスプレッドシートへのアクセスを確認
    async fn check_spreadsheet_exists(
        &self,
        sheets_client: &Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: &str,
    ) -> Result<(), ExternalServiceError> {
        // spreadsheets.get を呼び出してアクセス確認
        let result = sheets_client
            .spreadsheets()
            .get(spreadsheet_id)
            .doit()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("403") || error_msg.contains("Forbidden") {
                    Err(ExternalServiceError::GoogleSheetsApiError {
                        message: "サービスアカウントに閲覧権限がありません。Googleスプレッドシートの共有設定を確認してください".to_string(),
                    })
                } else if error_msg.contains("404") || error_msg.contains("Not found") {
                    Err(ExternalServiceError::SpreadsheetNotFound {
                        spreadsheet_url: spreadsheet_id.to_string(),
                    })
                } else {
                    Err(ExternalServiceError::GoogleSheetsApiError {
                        message: format!("スプレッドシートへのアクセスに失敗しました: {e}"),
                    })
                }
            }
        }
    }
}

#[async_trait]
impl<R, G, U> GuildSpreadsheetConfigServiceTrait for GuildSpreadsheetConfigService<R, G, U>
where
    R: GuildSpreadsheetConfigRepositoryTrait,
    G: GoogleAuthServiceTrait,
    U: SpreadsheetUrlServiceTrait,
{
    async fn verify_spreadsheet_access(
        &self,
        spreadsheet_id: &str,
    ) -> Result<(), ExternalServiceError> {
        // Google Sheets APIクライアントを取得
        let sheets_client = self.google_auth_service.get_sheets_client().await?;

        // アクセス確認
        self.check_spreadsheet_exists(&sheets_client, spreadsheet_id)
            .await
    }

    async fn register_spreadsheets(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        load_spreadsheet_id: &str,
        push_spreadsheet_id: &str,
    ) -> Result<(), BusinessRuleError> {
        // Repository層に委譲
        self.repository
            .upsert_import_spreadsheet_id(txn, guild_id, load_spreadsheet_id)
            .await
            .map_err(|e| BusinessRuleError::InvalidState {
                entity: "guild_spreadsheet_imports".to_string(),
                current_state: format!("登録失敗: {e}"),
            })?;

        self.repository
            .upsert_export_spreadsheet_id(txn, guild_id, push_spreadsheet_id)
            .await
            .map_err(|e| BusinessRuleError::InvalidState {
                entity: "guild_spreadsheet_exports".to_string(),
                current_state: format!("登録失敗: {e}"),
            })?;

        Ok(())
    }

    async fn get_import_spreadsheet_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<String>, BusinessRuleError> {
        self.repository
            .find_import_spreadsheet_id(db, guild_id)
            .await
            .map_err(|e| BusinessRuleError::InvalidState {
                entity: "guild_spreadsheet_imports".to_string(),
                current_state: format!("取得失敗: {e}"),
            })
    }

    async fn get_export_spreadsheet_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<String>, BusinessRuleError> {
        self.repository
            .find_export_spreadsheet_id(db, guild_id)
            .await
            .map_err(|e| BusinessRuleError::InvalidState {
                entity: "guild_spreadsheet_exports".to_string(),
                current_state: format!("取得失敗: {e}"),
            })
    }
}

#[cfg(test)]
mod tests {

    // 実際のGoogle API接続が必要なため、統合テストで実施
    // ここでは型チェックとトレイト実装の確認のみ
    #[test]
    fn test_service_trait_implementation() {
        // トレイト実装の型チェック
    }
}
