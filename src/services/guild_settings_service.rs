use crate::repository::GuildSettingsRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use tracing::debug;

/// ギルド設定情報
#[derive(Debug, Clone)]
pub struct GuildSettingsData {
    pub timezone: String,
    pub locale: String,
}

/// ギルド設定参照サービス
pub struct GuildSettingsService<R: GuildSettingsRepository> {
    repository: R,
}

impl<R: GuildSettingsRepository> GuildSettingsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// ギルド設定を取得（トランザクション内）
    pub async fn get_guild_settings_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<GuildSettingsData>> {
        let settings = self
            .repository
            .find_by_guild_id_with_txn(txn, guild_id)
            .await?;

        debug!(
            guild_id = guild_id,
            has_settings = settings.is_some(),
            "ギルド設定を取得しました"
        );

        Ok(settings.map(|s| GuildSettingsData {
            timezone: s.timezone,
            locale: s.locale,
        }))
    }
}
