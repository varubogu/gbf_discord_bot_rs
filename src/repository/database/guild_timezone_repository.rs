use crate::models::entities::guild_timezones;
use crate::types::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, Set};
use tracing::{debug, error, info};

/// guild_timezonesテーブルのRepository
pub struct GuildTimezoneRepository;

impl GuildTimezoneRepository {
    pub fn new() -> Self {
        Self
    }

    /// ギルドタイムゾーンを登録または更新（トランザクション内）
    pub async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        timezone: &str,
    ) -> Result<guild_timezones::Model> {
        debug!(
            guild_id = guild_id,
            timezone = timezone,
            "ギルドタイムゾーンを登録または更新します"
        );

        let now = chrono::Utc::now();

        // INSERT ... ON CONFLICT DO UPDATE を使用
        let active_model = guild_timezones::ActiveModel {
            guild_id: Set(guild_id),
            timezone: Set(timezone.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // UPSERTを実行（主キーが重複する場合は更新）
        guild_timezones::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(guild_timezones::Column::GuildId)
                    .update_columns([
                        guild_timezones::Column::Timezone,
                        guild_timezones::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドタイムゾーンのUPSERTに失敗しました"
                );
                e
            })?;

        // UPSERT後のデータを取得
        let model = guild_timezones::Entity::find_by_id(guild_id)
            .one(txn)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "ギルドタイムゾーンの取得に失敗しました: guild_id={}",
                    guild_id
                ))
            })?;

        info!(
            guild_id = guild_id,
            timezone = timezone,
            "ギルドタイムゾーンを登録または更新しました"
        );

        Ok(model)
    }

    /// ギルドIDでタイムゾーン設定を取得（トランザクションなし）
    pub async fn find_by_guild_id(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
    ) -> Result<Option<guild_timezones::Model>> {
        debug!(
            guild_id = guild_id,
            "ギルドタイムゾーンを取得します"
        );

        let model = guild_timezones::Entity::find_by_id(guild_id)
            .one(db)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドタイムゾーンの取得に失敗しました"
                );
                e
            })?;

        Ok(model)
    }

    /// ギルドIDでタイムゾーン設定を取得（トランザクション内）
    pub async fn find_by_guild_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<guild_timezones::Model>> {
        debug!(
            guild_id = guild_id,
            "ギルドタイムゾーンを取得します（トランザクション内）"
        );

        let model = guild_timezones::Entity::find_by_id(guild_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドタイムゾーンの取得に失敗しました"
                );
                e
            })?;

        Ok(model)
    }
}
