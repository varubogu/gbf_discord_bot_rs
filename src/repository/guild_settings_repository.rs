use crate::models::entities::guild_master::guild_settings;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

/// ギルド設定リポジトリの抽象インターフェース
#[async_trait]
pub trait GuildSettingsRepository: Send + Sync {
    /// ギルド設定（タイムゾーンとロケール）を登録または更新（トランザクション内）
    async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        timezone: &str,
        locale: &str,
    ) -> Result<guild_settings::Model>;

    /// ギルドIDで設定を取得（トランザクションなし）
    async fn find_by_guild_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<guild_settings::Model>>;

    /// ギルドIDで設定を取得（トランザクション内）
    async fn find_by_guild_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<guild_settings::Model>>;
}
