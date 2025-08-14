use async_trait::async_trait;
use crate::types::PoiseError;
use crate::models::environment::Environment;

/// 環境設定リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// 全環境設定を取得
    async fn get_all(&self) -> Result<Vec<Environment>, PoiseError>;
    
    /// キーで環境設定を取得
    async fn get_by_key(&self, key: &str) -> Result<Option<Environment>, PoiseError>;
    
    /// 環境変数を設定（存在しない場合は作成、存在する場合は更新）
    async fn set(&self, key: &str, value: &str) -> Result<Environment, PoiseError>;
}