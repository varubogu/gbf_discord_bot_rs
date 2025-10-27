use poise::serenity_prelude::{CreateEmbed, Message, ReactionType};
use thiserror::Error;

/// Discord操作を抽象化する列挙型
#[derive(Debug, Clone)]
pub enum DiscordOperation {
    SendMessage {
        channel_id: u64,
        content: String,
        embed: Option<CreateEmbed>,
    },
    EditMessage {
        channel_id: u64,
        message_id: u64,
        content: Option<String>,
        embed: Option<CreateEmbed>,
    },
    AddReaction {
        message: Message,
        emoji: ReactionType,
    },
    DeleteMessage {
        channel_id: u64,
        message_id: u64,
    },
    SendPrivateMessage {
        user_id: u64,
        content: String,
    },
}

/// Discord操作の結果
#[derive(Debug, Clone)]
pub struct DiscordOperationResult {
    pub message_id: u64,
    pub message: Option<Message>,
}

/// Discord操作エラー
#[derive(Error, Debug)]
pub enum DiscordOperationError {
    #[error("メッセージの送信に失敗しました: {0}")]
    MessageSendFailed(String),

    #[error("メッセージの編集に失敗しました: {0}")]
    MessageEditFailed(String),

    #[error("リアクションの追加に失敗しました: {0}")]
    ReactionAddFailed(String),

    #[error("権限が不足しています")]
    PermissionDenied,

    #[error("チャンネルが見つかりません")]
    ChannelNotFound,

    #[error("メッセージが見つかりません")]
    MessageNotFound,

    #[error("Discord API接続エラー: {0}")]
    ConnectionError(String),
}

impl From<poise::serenity_prelude::Error> for DiscordOperationError {
    fn from(err: poise::serenity_prelude::Error) -> Self {
        use poise::serenity_prelude::{Error as SerenityError, HttpError};

        match err {
            SerenityError::Http(HttpError::UnsuccessfulRequest(ref response)) => {
                match response.error.code {
                    50013 => DiscordOperationError::PermissionDenied,
                    10003 => DiscordOperationError::ChannelNotFound,
                    10008 => DiscordOperationError::MessageNotFound,
                    _ => DiscordOperationError::ConnectionError(err.to_string()),
                }
            }
            _ => DiscordOperationError::ConnectionError(err.to_string()),
        }
    }
}
