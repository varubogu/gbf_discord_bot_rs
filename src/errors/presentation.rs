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
                ExternalServiceError::GoogleSheetsApiError { message } => {
                    // UUID書き込み失敗のメッセージを検出
                    if message.contains("UUID書き戻しに失敗") {
                        format!(
                            "❌ スプレッドシートへのUUID書き込みに失敗しました\n\n\
                            【原因】\n\
                            サービスアカウントにスプレッドシートの編集権限がありません。\n\n\
                            【対処方法】\n\
                            1. Googleスプレッドシートを開く\n\
                            2. 右上の「共有」をクリック\n\
                            3. サービスアカウントのメールアドレスに「編集者」権限を付与\n\n\
                            ※データベースへの登録はロールバックされました。\n\
                            権限を付与した後、再度読み込みコマンドを実行してください。"
                        )
                    } else {
                        format!("🔧 Googleスプレッドシートへのアクセスに失敗しました。\n詳細: {}", message)
                    }
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
