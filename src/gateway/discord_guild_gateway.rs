//! Discordギルド操作Gatewayトレイト

use async_trait::async_trait;

use crate::errors::GatewayError;
use crate::types::discord::{DiscordGuildId, DiscordUserId, GuildEmoji, GuildMember, GuildRole};

/// Discordギルド操作を抽象化するトレイト
///
/// ギルド情報、メンバー、ロール、絵文字の取得を提供する。
/// ビジネスロジック層はこのトレイトを通じてギルド情報を取得する。
#[async_trait]
pub trait DiscordGuildGateway: Send + Sync {
    /// ギルドメンバーを取得する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    /// * `user_id` - ユーザーID
    ///
    /// # Returns
    ///
    /// ギルドメンバー情報
    async fn get_member(
        &self,
        guild_id: DiscordGuildId,
        user_id: DiscordUserId,
    ) -> Result<GuildMember, GatewayError>;

    /// ギルドロール一覧を取得する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    ///
    /// # Returns
    ///
    /// ギルドロール一覧
    async fn get_roles(&self, guild_id: DiscordGuildId) -> Result<Vec<GuildRole>, GatewayError>;

    /// ギルド絵文字一覧を取得する
    ///
    /// # Arguments
    ///
    /// * `guild_id` - ギルドID
    ///
    /// # Returns
    ///
    /// ギルド絵文字一覧
    async fn get_emojis(&self, guild_id: DiscordGuildId) -> Result<Vec<GuildEmoji>, GatewayError>;
}
