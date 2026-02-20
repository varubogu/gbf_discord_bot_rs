use crate::repository::GuildSettingsRepository;
use crate::types::Result;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use tracing::debug;

pub const DEFAULT_LOCALE: &str = "ja";

/// ロケール取得サービス
///
/// guild_settings.locale を参照し、未設定時は `ja` を返す。
pub struct LocaleService<R: GuildSettingsRepository> {
    repository: R,
}

impl<R: GuildSettingsRepository> LocaleService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// ギルドのロケールを取得（トランザクションなし）
    /// 未設定時はデフォルト（ja）を返す
    pub async fn get_guild_locale(&self, db: &DatabaseConnection, guild_id: i64) -> Result<String> {
        debug!(guild_id = guild_id, "ギルドのロケールを取得します");

        let settings = self.repository.find_by_guild_id(db, guild_id).await?;
        Ok(self.resolve_locale(guild_id, settings.map(|s| s.locale)))
    }

    /// ギルドのロケールを取得（トランザクション内）
    /// 未設定時はデフォルト（ja）を返す
    pub async fn get_guild_locale_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<String> {
        debug!(
            guild_id = guild_id,
            "ギルドのロケールを取得します（トランザクション内）"
        );

        let settings = self
            .repository
            .find_by_guild_id_with_txn(txn, guild_id)
            .await?;
        Ok(self.resolve_locale(guild_id, settings.map(|s| s.locale)))
    }

    fn resolve_locale(&self, guild_id: i64, locale_opt: Option<String>) -> String {
        match locale_opt {
            Some(locale) => locale,
            None => {
                debug!(
                    guild_id = guild_id,
                    "ロケール未設定のため、デフォルト（ja）を使用します"
                );
                DEFAULT_LOCALE.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct DummyGuildSettingsRepository;

    #[async_trait::async_trait]
    impl GuildSettingsRepository for DummyGuildSettingsRepository {
        async fn upsert_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
            _timezone: &str,
            _locale: &str,
        ) -> Result<crate::models::entities::guild_master::guild_settings::Model> {
            unimplemented!("このテストでは使用しません")
        }

        async fn find_by_guild_id(
            &self,
            _db: &DatabaseConnection,
            _guild_id: i64,
        ) -> Result<Option<crate::models::entities::guild_master::guild_settings::Model>> {
            unimplemented!("このテストでは使用しません")
        }

        async fn find_by_guild_id_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
        ) -> Result<Option<crate::models::entities::guild_master::guild_settings::Model>> {
            unimplemented!("このテストでは使用しません")
        }
    }

    #[test]
    fn test_resolve_locale_returns_default_when_not_set() {
        let service = LocaleService::new(DummyGuildSettingsRepository);
        let locale = service.resolve_locale(1, None);
        assert_eq!(locale, "ja");
    }

    #[test]
    fn test_resolve_locale_returns_value_when_set() {
        let service = LocaleService::new(DummyGuildSettingsRepository);
        let locale = service.resolve_locale(1, Some("en".to_string()));
        assert_eq!(locale, "en");
    }
}
