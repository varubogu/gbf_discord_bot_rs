//! Discordチャンネル操作Gatewayトレイト

use async_trait::async_trait;

use crate::errors::GatewayError;
use crate::types::discord::{
    ChannelCreateParams, ChannelData, ChannelEditParams, DiscordChannelId, DiscordGuildId,
};

/// Discordチャンネル操作を抽象化するトレイト
///
/// チャンネルの作成・編集・削除・取得を提供する。
/// ビジネスロジック層はこのトレイトを通じてDiscordチャンネルを操作する。
#[async_trait]
pub trait DiscordChannelGateway: Send + Sync {
    /// チャンネルを作成する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `params` - チャンネル作成パラメータ
    ///
    /// # Returns
    ///
    /// 作成されたチャンネルのID
    async fn create_channel(
        &self,
        guild_id: DiscordGuildId,
        params: ChannelCreateParams,
    ) -> Result<DiscordChannelId, GatewayError>;

    /// チャンネルを編集する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - 編集対象チャンネルID
    /// * `params` - チャンネル編集パラメータ
    async fn edit_channel(
        &self,
        channel_id: DiscordChannelId,
        params: ChannelEditParams,
    ) -> Result<(), GatewayError>;

    /// チャンネルを削除する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - 削除対象チャンネルID
    async fn delete_channel(&self, channel_id: DiscordChannelId) -> Result<(), GatewayError>;

    /// チャンネル情報を取得する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - 取得対象チャンネルID
    ///
    /// # Returns
    ///
    /// チャンネルデータ
    async fn get_channel(&self, channel_id: DiscordChannelId) -> Result<ChannelData, GatewayError>;
}
