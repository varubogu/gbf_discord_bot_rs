//! Gateway層エラー型
//!
//! Discord Gateway操作で発生するエラーを定義する。
//! poise/serenityのエラーをラップし、ビジネスロジック層から直接依存を排除する。

use thiserror::Error;

/// Gateway層で発生するエラー
#[derive(Debug, Error)]
pub enum GatewayError {
    /// メッセージ送信に失敗した
    #[error("メッセージ送信に失敗しました: {0}")]
    SendMessageFailed(String),

    /// メッセージ編集に失敗した
    #[error("メッセージ編集に失敗しました: {0}")]
    EditMessageFailed(String),

    /// メッセージ削除に失敗した
    #[error("メッセージ削除に失敗しました: {0}")]
    DeleteMessageFailed(String),

    /// メッセージ取得に失敗した
    #[error("メッセージ取得に失敗しました: {0}")]
    GetMessageFailed(String),

    /// チャンネル作成に失敗した
    #[error("チャンネル作成に失敗しました: {0}")]
    CreateChannelFailed(String),

    /// チャンネル編集に失敗した
    #[error("チャンネル編集に失敗しました: {0}")]
    EditChannelFailed(String),

    /// チャンネル削除に失敗した
    #[error("チャンネル削除に失敗しました: {0}")]
    DeleteChannelFailed(String),

    /// チャンネル取得に失敗した
    #[error("チャンネル取得に失敗しました: {0}")]
    GetChannelFailed(String),

    /// インタラクション遅延応答に失敗した
    #[error("インタラクション遅延応答に失敗しました: {0}")]
    DeferInteractionFailed(String),

    /// インタラクション応答に失敗した
    #[error("インタラクション応答に失敗しました: {0}")]
    RespondToInteractionFailed(String),

    /// インタラクション応答編集に失敗した
    #[error("インタラクション応答編集に失敗しました: {0}")]
    EditInteractionResponseFailed(String),

    /// リアクション取得に失敗した
    #[error("リアクション取得に失敗しました: {0}")]
    GetReactionsFailed(String),

    /// リアクション追加に失敗した
    #[error("リアクション追加に失敗しました: {0}")]
    AddReactionFailed(String),

    /// ギルドメンバー取得に失敗した
    #[error("ギルドメンバー取得に失敗しました: {0}")]
    GetMemberFailed(String),

    /// ギルドロール取得に失敗した
    #[error("ギルドロール取得に失敗しました: {0}")]
    GetRolesFailed(String),

    /// ギルド絵文字取得に失敗した
    #[error("ギルド絵文字取得に失敗しました: {0}")]
    GetEmojisFailed(String),

    /// HTTPクライアントが利用できない
    #[error("HTTPクライアントが利用できません")]
    HttpClientUnavailable,

    /// 権限エラー
    #[error("権限が不足しています: {0}")]
    PermissionDenied(String),

    /// レート制限エラー
    #[error("レート制限に達しました: {0}")]
    RateLimited(String),

    /// リソースが見つからない
    #[error("リソースが見つかりません: {0}")]
    NotFound(String),

    /// 内部エラー
    #[error("内部エラーが発生しました: {0}")]
    Internal(String),
}

impl GatewayError {
    /// Discord APIエラーからメッセージ送信失敗エラーを作成する
    pub fn send_message_failed(error: impl std::fmt::Display) -> Self {
        Self::SendMessageFailed(error.to_string())
    }

    /// Discord APIエラーからメッセージ編集失敗エラーを作成する
    pub fn edit_message_failed(error: impl std::fmt::Display) -> Self {
        Self::EditMessageFailed(error.to_string())
    }

    /// Discord APIエラーからメッセージ削除失敗エラーを作成する
    pub fn delete_message_failed(error: impl std::fmt::Display) -> Self {
        Self::DeleteMessageFailed(error.to_string())
    }

    /// Discord APIエラーからメッセージ取得失敗エラーを作成する
    pub fn get_message_failed(error: impl std::fmt::Display) -> Self {
        Self::GetMessageFailed(error.to_string())
    }

    /// Discord APIエラーからチャンネル作成失敗エラーを作成する
    pub fn create_channel_failed(error: impl std::fmt::Display) -> Self {
        Self::CreateChannelFailed(error.to_string())
    }

    /// Discord APIエラーからチャンネル編集失敗エラーを作成する
    pub fn edit_channel_failed(error: impl std::fmt::Display) -> Self {
        Self::EditChannelFailed(error.to_string())
    }

    /// Discord APIエラーからチャンネル削除失敗エラーを作成する
    pub fn delete_channel_failed(error: impl std::fmt::Display) -> Self {
        Self::DeleteChannelFailed(error.to_string())
    }

    /// Discord APIエラーからチャンネル取得失敗エラーを作成する
    pub fn get_channel_failed(error: impl std::fmt::Display) -> Self {
        Self::GetChannelFailed(error.to_string())
    }

    /// Discord APIエラーからインタラクション遅延応答失敗エラーを作成する
    pub fn defer_interaction_failed(error: impl std::fmt::Display) -> Self {
        Self::DeferInteractionFailed(error.to_string())
    }

    /// Discord APIエラーからインタラクション応答失敗エラーを作成する
    pub fn respond_to_interaction_failed(error: impl std::fmt::Display) -> Self {
        Self::RespondToInteractionFailed(error.to_string())
    }

    /// Discord APIエラーからインタラクション応答編集失敗エラーを作成する
    pub fn edit_interaction_response_failed(error: impl std::fmt::Display) -> Self {
        Self::EditInteractionResponseFailed(error.to_string())
    }

    /// Discord APIエラーからリアクション取得失敗エラーを作成する
    pub fn get_reactions_failed(error: impl std::fmt::Display) -> Self {
        Self::GetReactionsFailed(error.to_string())
    }

    /// Discord APIエラーからリアクション追加失敗エラーを作成する
    pub fn add_reaction_failed(error: impl std::fmt::Display) -> Self {
        Self::AddReactionFailed(error.to_string())
    }

    /// Discord APIエラーからギルドメンバー取得失敗エラーを作成する
    pub fn get_member_failed(error: impl std::fmt::Display) -> Self {
        Self::GetMemberFailed(error.to_string())
    }

    /// Discord APIエラーからギルドロール取得失敗エラーを作成する
    pub fn get_roles_failed(error: impl std::fmt::Display) -> Self {
        Self::GetRolesFailed(error.to_string())
    }

    /// Discord APIエラーからギルド絵文字取得失敗エラーを作成する
    pub fn get_emojis_failed(error: impl std::fmt::Display) -> Self {
        Self::GetEmojisFailed(error.to_string())
    }

    /// リソースが見つからないエラーを作成する
    pub fn not_found(resource: impl std::fmt::Display) -> Self {
        Self::NotFound(resource.to_string())
    }

    /// 権限エラーを作成する
    pub fn permission_denied(message: impl std::fmt::Display) -> Self {
        Self::PermissionDenied(message.to_string())
    }

    /// レート制限エラーを作成する
    pub fn rate_limited(message: impl std::fmt::Display) -> Self {
        Self::RateLimited(message.to_string())
    }

    /// 内部エラーを作成する
    pub fn internal(message: impl std::fmt::Display) -> Self {
        Self::Internal(message.to_string())
    }

    /// 再試行可能かどうかを判定する
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited(_) | Self::Internal(_))
    }

    /// 権限エラーかどうかを判定する
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// リソースが見つからないエラーかどうかを判定する
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_error_display() {
        let error = GatewayError::SendMessageFailed("Connection refused".to_string());
        assert_eq!(
            format!("{}", error),
            "メッセージ送信に失敗しました: Connection refused"
        );
    }

    #[test]
    fn test_gateway_error_factory_methods() {
        let error = GatewayError::send_message_failed("test error");
        assert!(matches!(error, GatewayError::SendMessageFailed(_)));

        let error = GatewayError::not_found("channel:12345");
        assert!(error.is_not_found());

        let error = GatewayError::permission_denied("Missing SEND_MESSAGES");
        assert!(error.is_permission_error());
    }

    #[test]
    fn test_is_retryable() {
        let rate_limited = GatewayError::RateLimited("1000ms".to_string());
        assert!(rate_limited.is_retryable());

        let not_found = GatewayError::NotFound("message".to_string());
        assert!(!not_found.is_retryable());
    }
}
