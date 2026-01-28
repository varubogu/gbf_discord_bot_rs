//! Discordメッセージ操作Gatewayトレイト

use async_trait::async_trait;

use crate::errors::GatewayError;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, MessageContent, MessageData};

/// Discordメッセージ操作を抽象化するトレイト
///
/// メッセージの送信・編集・削除・取得を提供する。
/// ビジネスロジック層はこのトレイトを通じてDiscordメッセージを操作する。
#[async_trait]
pub trait DiscordMessageGateway: Send + Sync {
    /// メッセージを送信し、送信されたメッセージのIDを返す
    ///
    /// # Arguments
    ///
    /// * `channel_id` - 送信先チャンネルID
    /// * `content` - メッセージコンテンツ
    ///
    /// # Returns
    ///
    /// 送信されたメッセージのID
    async fn send_message(
        &self,
        channel_id: DiscordChannelId,
        content: MessageContent,
    ) -> Result<DiscordMessageId, GatewayError>;

    /// メッセージを編集する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - 編集対象メッセージID
    /// * `content` - 新しいメッセージコンテンツ
    async fn edit_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        content: MessageContent,
    ) -> Result<(), GatewayError>;

    /// メッセージを削除する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - 削除対象メッセージID
    async fn delete_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<(), GatewayError>;

    /// メッセージを取得する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `message_id` - 取得対象メッセージID
    ///
    /// # Returns
    ///
    /// メッセージデータ
    async fn get_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<MessageData, GatewayError>;

    /// チャンネル内のメッセージ一覧を取得する
    ///
    /// # Arguments
    ///
    /// * `channel_id` - チャンネルID
    /// * `limit` - 取得する最大件数
    ///
    /// # Returns
    ///
    /// メッセージデータ一覧（新しい順）
    async fn get_messages(
        &self,
        channel_id: DiscordChannelId,
        limit: u8,
    ) -> Result<Vec<MessageData>, GatewayError>;
}
