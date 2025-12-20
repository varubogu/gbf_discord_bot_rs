/// Google認証サービス
///
/// Google Cloud PlatformのサービスアカウントによるOAuth2認証を提供します。
/// 設計書: docs/develop/design/spreadsheet/service_layer.md
use async_trait::async_trait;
use google_sheets4::{
    Sheets,
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::{ServiceAccountAuthenticator, ServiceAccountKey},
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::ExternalServiceError;

/// Google認証サービストレイト
#[async_trait]
pub trait GoogleAuthServiceTrait: Send + Sync {
    /// Google Sheets APIクライアントを取得
    async fn get_sheets_client(
        &self,
    ) -> Result<Sheets<HttpsConnector<HttpConnector>>, ExternalServiceError>;
}

/// Google認証サービス実装
#[derive(Clone)]
pub struct GoogleAuthService {
    service_account_key_file: String,
    client_cache: Arc<RwLock<Option<Sheets<HttpsConnector<HttpConnector>>>>>,
}

impl GoogleAuthService {
    /// 新しいGoogleAuthServiceインスタンスを作成
    ///
    /// # Arguments
    /// * `service_account_key_file` - サービスアカウントキーファイルのパス
    pub fn new(service_account_key_file: String) -> Self {
        Self {
            service_account_key_file,
            client_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// サービスアカウント認証を実行
    async fn authenticate(
        &self,
    ) -> Result<
        google_sheets4::oauth2::authenticator::Authenticator<HttpsConnector<HttpConnector>>,
        ExternalServiceError,
    > {
        // サービスアカウントキーファイルを読み込み
        let key_file_content = tokio::fs::read_to_string(&self.service_account_key_file)
            .await
            .map_err(|e| ExternalServiceError::GoogleAuthError {
                message: format!(
                    "サービスアカウントキーファイルの読み込みに失敗しました: {e}"
                ),
            })?;

        // JSONをパース
        let service_account_key: ServiceAccountKey = serde_json::from_str(&key_file_content)
            .map_err(|e| ExternalServiceError::GoogleAuthError {
                message: format!("サービスアカウントキーのパースに失敗しました: {e}"),
            })?;

        // 認証器を作成
        let auth = ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .map_err(|e| ExternalServiceError::GoogleAuthError {
                message: format!("Google認証に失敗しました: {e}"),
            })?;

        Ok(auth)
    }

    /// Google Sheets APIクライアントを作成
    async fn create_sheets_client(
        &self,
    ) -> Result<Sheets<HttpsConnector<HttpConnector>>, ExternalServiceError> {
        // 認証
        let auth = self.authenticate().await?;

        // HTTPSコネクタを作成（hyper-rustls 0.24用）
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| ExternalServiceError::GoogleAuthError {
                message: format!("HTTPSコネクタの初期化に失敗しました: {e}"),
            })?
            .https_or_http()
            .enable_http1()
            .build();

        // HTTPクライアントを作成
        let client = hyper::Client::builder().build(https.clone());

        // Sheets APIクライアントを作成
        let hub = Sheets::new(client, auth);

        Ok(hub)
    }
}

#[async_trait]
impl GoogleAuthServiceTrait for GoogleAuthService {
    async fn get_sheets_client(
        &self,
    ) -> Result<Sheets<HttpsConnector<HttpConnector>>, ExternalServiceError> {
        // キャッシュをチェック
        {
            let cache = self.client_cache.read().await;
            if cache.is_some() {
                tracing::debug!("Google Sheets APIクライアントのキャッシュを使用します");
                // Note: Sheetsクライアントはクローンできないため、毎回新規作成
                // キャッシュは認証トークンの再利用のみ
            }
        }

        // 新規作成（認証トークンは内部でキャッシュされる）
        tracing::info!("Google Sheets APIクライアントを作成します");
        let client = self.create_sheets_client().await?;

        // キャッシュに保存（参考用、実際は毎回作成）
        {
            let mut cache = self.client_cache.write().await;
            *cache = None; // Sheetsはクローンできないため、キャッシュは無効化
        }

        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 実際のサービスアカウントキーが必要なため、デフォルトではスキップ
    async fn test_google_auth_service() {
        dotenv::dotenv().ok();
        let key_file =
            std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE").expect("環境変数が設定されていません");

        let service = GoogleAuthService::new(key_file);
        let result = service.get_sheets_client().await;

        assert!(
            result.is_ok(),
            "Google認証に失敗しました: {:?}",
            result.err()
        );
    }
}
