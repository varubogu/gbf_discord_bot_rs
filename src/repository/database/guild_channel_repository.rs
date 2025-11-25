use crate::models::entities::guild_channels;
use crate::types::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set};
use tracing::{debug, error, info};

/// guild_channelsテーブルのRepository
pub struct GuildChannelRepository {
    db: DatabaseConnection,
}

impl GuildChannelRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(
                error = %e,
                guild_id = guild_id,
                channel_type = channel_type,
                "ギルドチャンネルの登録に失敗しました"
            );
            e
        })?;

        info!(
            guild_id = guild_id,
            channel_type = channel_type,
            channel_id = channel_id,
            "ギルドチャンネルを登録しました"
        );

        Ok(model)
    }

    /// ギルドIDとチャンネル種別でギルドチャンネルを取得
    pub async fn get_by_guild_and_type(
        &self,
        guild_id: i64,
        channel_type: i32,
    ) -> Result<Option<guild_channels::Model>> {
        debug!(
            guild_id = guild_id,
            channel_type = channel_type,
            "ギルドチャンネルを取得します"
        );

        let model = guild_channels::Entity::find_by_id((guild_id, channel_type))
            .one(&self.db)
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
}
