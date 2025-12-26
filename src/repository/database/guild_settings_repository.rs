use crate::models::entities::guild_master::guild_settings;
use crate::types::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, Set};
use tracing::{debug, error, info};

/// guild_settingsテーブルのRepository
pub struct GuildSettingsRepository;

impl Default for GuildSettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildSettingsRepository {
    pub fn new() -> Self {
        Self
    }

    /// ギルド設定（タイムゾーンとロケール）を登録または更新（トランザクション内）
    pub async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        timezone: &str,
        locale: &str,
    ) -> Result<guild_settings::Model> {
        debug!(
            guild_id = guild_id,
            timezone = timezone,
            locale = locale,
            "ギルド設定を登録または更新します"
        );

        let now = chrono::Utc::now();

        // INSERT ... ON CONFLICT DO UPDATE を使用
        let active_model = guild_settings::ActiveModel {
            guild_id: Set(guild_id),
            timezone: Set(timezone.to_string()),
            locale: Set(locale.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // UPSERTを実行（主キーが重複する場合は更新）
        guild_settings::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(guild_settings::Column::GuildId)
                    .update_columns([
                        guild_settings::Column::Timezone,
                        guild_settings::Column::Locale,
                        guild_settings::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルド設定のUPSERTに失敗しました"
                );
                e
            })?;

        // UPSERT後のデータを取得
        let model = guild_settings::Entity::find_by_id(guild_id)
            .one(txn)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "ギルド設定の取得に失敗しました: guild_id={guild_id}"
                ))
            })?;

        info!(
            guild_id = guild_id,
            timezone = timezone,
            locale = locale,
            "ギルド設定を登録または更新しました"
        );

        Ok(model)
    }

    /// ギルドIDで設定を取得（トランザクションなし）
    pub async fn find_by_guild_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<guild_settings::Model>> {
        debug!(guild_id = guild_id, "ギルド設定を取得します");

        let model = guild_settings::Entity::find_by_id(guild_id)
            .one(db)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルド設定の取得に失敗しました"
                );
                e
            })?;

        Ok(model)
    }

    /// ギルドIDで設定を取得（トランザクション内）
    pub async fn find_by_guild_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<guild_settings::Model>> {
        debug!(
            guild_id = guild_id,
            "ギルド設定を取得します（トランザクション内）"
        );

        let model = guild_settings::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルド設定の取得に失敗しました"
                );
                e
            })?;

        Ok(model)
    }
}
