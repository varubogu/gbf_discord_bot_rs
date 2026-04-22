use crate::models::environments::Environments;
use async_trait::async_trait;
use sea_orm::DbErr;

/// グローバル環境変数リポジトリの抽象インターフェース
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// 全てのグローバル環境変数を取得
    async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// キーでグローバル環境変数を取得
    async fn get_by_key<'c, C>(&self, db: &'c C, key: &str) -> Result<Option<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// グローバル環境変数を設定
    async fn set<'c, C>(&self, db: &'c C, key: &str, value: &str) -> Result<Environments, DbErr>
    where
        C: sea_orm::ConnectionTrait;
}
