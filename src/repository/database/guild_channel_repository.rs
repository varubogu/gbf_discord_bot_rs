use crate::models::entities::guild_channels;
use crate::types::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error, info};

/// guild_channelsテーブルのRepository
pub struct GuildChannelRepository;

impl GuildChannelRepository {
    pub fn new() -> Self {
        Self
    }

    /// ギルドチャンネルを登録または更新（トランザクション内）
    pub async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
        channel_id: i64,
    ) -> Result<guild_channels::Model> {
        debug!(
            guild_id = guild_id,
            channel_type = channel_type,
            channel_id = channel_id,
            "ギルドチャンネルを登録または更新します"
        );

        let now = chrono::Utc::now();

        // INSERT ... ON CONFLICT DO UPDATE を使用
        let active_model = guild_channels::ActiveModel {
            guild_id: Set(guild_id),
            channel_type: Set(channel_type),
            channel_id: Set(channel_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // UPSERTを実行（主キーが重複する場合は更新）
        guild_channels::Entity::insert(active_model)
            .on_conflict(
                OnConflict::columns([
                    guild_channels::Column::GuildId,
                    guild_channels::Column::ChannelType,
                ])
                .update_columns([
                    guild_channels::Column::ChannelId,
                    guild_channels::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    channel_type = channel_type,
                    "ギルドチャンネルのUPSERTに失敗しました"
                );
                e
            })?;

        // UPSERT後のデータを取得
        let model = guild_channels::Entity::find_by_id((guild_id, channel_type))
            .one(txn)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "ギルドチャンネルの取得に失敗しました: guild_id={}, channel_type={}",
                    guild_id, channel_type
                ))
            })?;

        info!(
            guild_id = guild_id,
            channel_type = channel_type,
            channel_id = channel_id,
            "ギルドチャンネルを登録または更新しました"
        );

        Ok(model)
    }

    /// ギルドIDとチャンネル種別でギルドチャンネルを取得（トランザクション内）
    pub async fn get_by_guild_and_type_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
    ) -> Result<Option<guild_channels::Model>> {
        debug!(
            guild_id = guild_id,
            channel_type = channel_type,
            "ギルドチャンネルを取得します（トランザクション内）"
        );

        let model = guild_channels::Entity::find_by_id((guild_id, channel_type))
            .one(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    channel_type = channel_type,
                    "ギルドチャンネルの取得に失敗しました"
                );
                e
            })?;

        Ok(model)
    }

    /// ギルドIDでギルドチャンネル一覧を取得（トランザクション内）
    pub async fn get_all_by_guild_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<guild_channels::Model>> {
        debug!(
            guild_id = guild_id,
            "ギルドチャンネル一覧を取得します（トランザクション内）"
        );

        let models = guild_channels::Entity::find()
            .filter(guild_channels::Column::GuildId.eq(guild_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドチャンネル一覧の取得に失敗しました"
                );
                e
            })?;

        Ok(models)
    }

    /// ギルドチャンネルを削除（トランザクション内）
    pub async fn delete_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type: i32,
    ) -> Result<()> {
        debug!(
            guild_id = guild_id,
            channel_type = channel_type,
            "ギルドチャンネルを削除します（トランザクション内）"
        );

        let result = guild_channels::Entity::delete_by_id((guild_id, channel_type))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    channel_type = channel_type,
                    "ギルドチャンネルの削除に失敗しました"
                );
                e
            })?;

        if result.rows_affected == 0 {
            return Err(crate::types::AppError::NotFound(format!(
                "削除対象のギルドチャンネルが見つかりませんでした: guild_id={}, channel_type={}",
                guild_id, channel_type
            )));
        }

        info!(
            guild_id = guild_id,
            channel_type = channel_type,
            "ギルドチャンネルを削除しました"
        );

        Ok(())
    }
}
