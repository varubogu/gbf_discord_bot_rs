use crate::errors::RepositoryError;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// ギルドスプレッドシート設定リポジトリのトレイト
#[async_trait]
pub trait GuildSpreadsheetConfigRepositoryTrait: Send + Sync {
    /// 読み込み用スプレッドシートIDを取得
    async fn find_import_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait;

    /// 書き込み用スプレッドシートIDを取得
    async fn find_export_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait;

    /// 読み込み用スプレッドシートIDを登録/更新（トランザクション版）
    async fn upsert_import_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError>;

    /// 書き込み用スプレッドシートIDを登録/更新（トランザクション版）
    async fn upsert_export_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError>;
}
