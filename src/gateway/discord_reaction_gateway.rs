//! Discordリアクション操作Gatewayトレイト

use async_trait::async_trait;

use crate::errors::GatewayError;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, DiscordUserId, ReactionEmoji};

/// Discordリアクション操作を抽象化するトレイト
///
/// リアクションの追加・取得を提供する。
/// ビジネスロジック層はこのトレイトを通じてリアクションを操作する。
#[async_trait]
pub trait DiscordReactionGateway: Send + Sync {
    /// リアクションしたユーザー一覧を取得する
    ///
    /// ボットユーザーは結果から除外される。
    /// これはbot自身が募集メッセージにリアクションを追加するため、
    /// 参加者一覧に含まれないようにするためである。
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - メッセージID
    /// * `emoji` - リアクション絵文字
    /// * `limit` - 取得する最大件数（None の場合はデフォルト値）
    ///
    /// # Returns
    ///
    /// リアクションしたユーザーID一覧（ボットユーザーを除く）
    async fn get_reaction_users(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
        limit: Option<u8>,
    ) -> Result<Vec<DiscordUserId>, GatewayError>;

    /// リアクションを追加する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - メッセージID
    /// * `emoji` - 追加する絵文字
    async fn add_reaction(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
    ) -> Result<(), GatewayError>;

    /// リアクションを削除する
    ///
    /// 自身のリアクションを削除する。
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - メッセージID
    /// * `emoji` - 削除する絵文字
    async fn remove_own_reaction(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
    ) -> Result<(), GatewayError>;
}
