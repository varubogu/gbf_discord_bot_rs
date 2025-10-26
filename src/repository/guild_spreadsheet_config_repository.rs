/// ギルドスプレッドシート設定リポジトリ
///
/// guild_spreadsheet_imports と guild_spreadsheet_exports テーブルへのアクセスを提供

use crate::errors::RepositoryError;
use crate::models::entities::{guild_spreadsheet_exports, guild_spreadsheet_imports};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter,
};

/// ギルドスプレッドシート設定リポジトリのトレイト
#[async_trait]
pub trait GuildSpreadsheetConfigRepositoryTrait: Send + Sync {
    /// 読み込み用スプレッドシートIDを取得
    async fn find_import_spreadsheet_id(
        &self,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>;

    /// 書き込み用スプレッドシートIDを取得
    async fn find_export_spreadsheet_id(
        &self,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>;

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

/// ギルドスプレッドシート設定リポジトリの実装
#[derive(Clone)]
pub struct GuildSpreadsheetConfigRepository {
    db: DatabaseConnection,
}

impl GuildSpreadsheetConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl GuildSpreadsheetConfigRepositoryTrait for GuildSpreadsheetConfigRepository {
    async fn find_import_spreadsheet_id(
        &self,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError> {
        let result = guild_spreadsheet_imports::Entity::find()
            .filter(guild_spreadsheet_imports::Column::GuildId.eq(guild_id))
            .one(&self.db)
            .await?;

        Ok(result.map(|model| model.spreadsheet_id))
    }

    async fn find_export_spreadsheet_id(
        &self,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError> {
        let result = guild_spreadsheet_exports::Entity::find()
            .filter(guild_spreadsheet_exports::Column::GuildId.eq(guild_id))
            .one(&self.db)
            .await?;

        Ok(result.map(|model| model.spreadsheet_id))
    }

    async fn upsert_import_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();

        let active_model = guild_spreadsheet_imports::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        // INSERT を試行
        let insert_result = active_model.insert(txn).await;

        // INSERT が失敗した場合（既存レコード）、UPDATE を実行
        if insert_result.is_err() {
            let update_model = guild_spreadsheet_imports::ActiveModel {
                guild_id: ActiveValue::Unchanged(guild_id),
                spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::Set(now),
            };

            update_model.update(txn).await?;
        }

        Ok(())
    }

    async fn upsert_export_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();

        let active_model = guild_spreadsheet_exports::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        // INSERT を試行
        let insert_result = active_model.insert(txn).await;

        // INSERT が失敗した場合（既存レコード）、UPDATE を実行
        if insert_result.is_err() {
            let update_model = guild_spreadsheet_exports::ActiveModel {
                guild_id: ActiveValue::Unchanged(guild_id),
                spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::Set(now),
            };

            update_model.update(txn).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 実際のDB接続が必要なため、統合テストで実施
    // ここでは型チェックとトレイト実装の確認のみ
    #[test]
    fn test_repository_trait_implementation() {
        fn assert_impl<T: GuildSpreadsheetConfigRepositoryTrait>() {}
        assert_impl::<GuildSpreadsheetConfigRepository>();
    }
}
