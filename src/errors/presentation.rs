/// Presentation層エラー
///
/// Discordユーザー向けエラーメッセージ。

use thiserror::Error;

use super::{facade::FacadeError, service::ExternalServiceError};

#[derive(Error, Debug)]
pub enum PresentationError {
    #[error("{message}")]
    UserFacingError {
        message: String,
        #[source]
        source: Option<FacadeError>,
    },
}

impl From<FacadeError> for PresentationError {
    fn from(err: FacadeError) -> Self {
        let message = match &err {
            FacadeError::Validation { source } => {
                format!("❌ 入力エラー: {}", source)
            }
            FacadeError::BusinessRule { source } => {
                format!("⚠️ 操作できません: {}", source)
            }
            FacadeError::ExternalService { source } => match source {
                ExternalServiceError::ServiceTimeout { .. } => {
                    "🔧 タイムアウトが発生しました。しばらく待ってから再試行してください。"
                        .to_string()
                }
                ExternalServiceError::GoogleSheetsApiError { .. } => {
                    "🔧 Googleスプレッドシートへのアクセスに失敗しました。".to_string()
                }
                ExternalServiceError::GoogleAuthError { .. } => {
                    "🔧 Google認証に失敗しました。管理者に連絡してください。".to_string()
                }
                ExternalServiceError::SpreadsheetNotFound { .. } => {
                    "🔧 スプレッドシートが見つかりません。URLを確認してください。".to_string()
                }
                _ => "🔧 外部サービスでエラーが発生しました。".to_string(),
            },
            FacadeError::Repository { .. } => {
                "🔧 データベースエラーが発生しました。管理者に連絡してください。".to_string()
            }
            FacadeError::TransactionError { .. } => {
                "🔧 処理に失敗しました。再試行してください。".to_string()
            }
            FacadeError::Initialization { message } => {
                format!("🔧 初期化エラー: {}", message)
            }
            FacadeError::Database { .. } => {
                "🔧 データベースエラーが発生しました。管理者に連絡してください。".to_string()
            }
        };

        PresentationError::UserFacingError {
            message,
            source: Some(err),
        }
    }
}
