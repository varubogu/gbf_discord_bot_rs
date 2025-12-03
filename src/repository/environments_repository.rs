use crate::models::environments::Environments;
use async_trait::async_trait;
use sea_orm::DbErr;

/// 環境設定リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// 全環境設定を取得
    async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// キーで環境設定を取得
    async fn get_by_key<'c, C>(&self, db: &'c C, key: &str) -> Result<Option<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// 環境変数を設定（存在しない場合は作成、存在する場合は更新）
    async fn set<'c, C>(&self, db: &'c C, key: &str, value: &str) -> Result<Environments, DbErr>
    where
        C: sea_orm::ConnectionTrait;
}
