use crate::models::entities::{master::channel_types, guild_master::guild_channels};
use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::guild_repository::GuildRepository;
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use tracing::{debug, info};

/// チャンネル管理Service
/// チャンネル登録・削除のビジネスロジックの責務を持つ
pub struct ChannelManagementService;

impl Default for ChannelManagementService {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelManagementService {
    pub fn new() -> Self {
        Self
    }

    /// ギルドを登録または更新
    pub async fn register_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        guild_name: String,
    ) -> Result<()> {
        let guild_repo = GuildRepository::new();
        guild_repo
            .upsert_with_txn(txn, guild_id, guild_name)
            .await?;

        debug!(guild_id = guild_id, "ギルドを登録しました");
        Ok(())
    }

    /// チャンネル種別をIDで取得
    pub async fn get_channel_type_by_id(
        &self,
        txn: &DatabaseTransaction,
        channel_type_id: i32,
    ) -> Result<channel_types::Model> {
        let channel_type_repo = ChannelTypeRepository::new();
        let channel_type = channel_type_repo
            .get_by_id(txn, channel_type_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "チャンネル種別ID {channel_type_id} が見つかりませんでした"
                ))
            })?;

        debug!(
            channel_type_id = channel_type_id,
            channel_type_name = %channel_type.name,
            "チャンネル種別を取得しました"
        );

        Ok(channel_type)
    }

    /// ギルドチャンネルを登録または更新
    pub async fn register_guild_channel(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type_id: i32,
        channel_id: i64,
    ) -> Result<()> {
        let guild_channel_repo = GuildChannelRepository::new();
        guild_channel_repo
            .upsert_with_txn(txn, guild_id, channel_type_id, channel_id)
            .await?;

        info!(
            guild_id = guild_id,
            channel_type_id = channel_type_id,
            channel_id = channel_id,
            "ギルドチャンネルを登録しました"
        );

        Ok(())
    }

    /// ギルドチャンネルを取得
    pub async fn get_guild_channel(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type_id: i32,
    ) -> Result<Option<guild_channels::Model>> {
        let guild_channel_repo = GuildChannelRepository::new();
        let channel = guild_channel_repo
            .get_by_guild_and_type_with_txn(txn, guild_id, channel_type_id)
            .await?;

        debug!(
            guild_id = guild_id,
            channel_type_id = channel_type_id,
            found = channel.is_some(),
            "ギルドチャンネルを取得しました"
        );

        Ok(channel)
    }

    /// ギルドチャンネルを削除
    pub async fn delete_guild_channel(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_type_id: i32,
    ) -> Result<()> {
        let guild_channel_repo = GuildChannelRepository::new();
        guild_channel_repo
            .delete_with_txn(txn, guild_id, channel_type_id)
            .await?;

        info!(
            guild_id = guild_id,
            channel_type_id = channel_type_id,
            "ギルドチャンネルを削除しました"
        );

        Ok(())
    }
}
