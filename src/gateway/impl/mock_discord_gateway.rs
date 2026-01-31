//! テスト用MockDiscord Gateway
//!
//! mockallを使用したモックGateway実装。
//! ユニットテストやインテグレーションテストで使用する。

use async_trait::async_trait;
use mockall::mock;

use crate::errors::GatewayError;
use crate::gateway::{
    DiscordChannelGateway, DiscordGuildGateway, DiscordInteractionGateway, DiscordMessageGateway,
    DiscordReactionGateway,
};
use crate::types::discord::{
    ChannelCreateParams, ChannelData, ChannelEditParams, DiscordChannelId, DiscordGuildId,
    DiscordInteractionId, DiscordMessageId, DiscordUserId, GuildEmoji, GuildMember, GuildRole,
    InteractionResponse, MessageContent, MessageData, ReactionEmoji,
};

mock! {
    /// テスト用モックDiscord Gateway
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crate::gateway::impl::mock_discord_gateway::MockDiscordGateway;
    ///
    /// let mut mock = MockDiscordGateway::new();
    ///
    /// mock.expect_send_message()
    ///     .returning(|_, _| Ok(DiscordMessageId::new(12345)));
    ///
    /// let service = NotificationService::new(Arc::new(mock));
    /// ```
    pub DiscordGateway {}

    #[async_trait]
    impl DiscordMessageGateway for DiscordGateway {
        async fn send_message(
            &self,
            channel_id: DiscordChannelId,
            content: MessageContent,
        ) -> Result<DiscordMessageId, GatewayError>;

        async fn edit_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
            content: MessageContent,
        ) -> Result<(), GatewayError>;

        async fn delete_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
        ) -> Result<(), GatewayError>;

        async fn get_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
        ) -> Result<MessageData, GatewayError>;

        async fn get_messages(
            &self,
            channel_id: DiscordChannelId,
            limit: u8,
        ) -> Result<Vec<MessageData>, GatewayError>;

        async fn send_reply(
            &self,
            channel_id: DiscordChannelId,
            reply_to_message_id: DiscordMessageId,
            content: MessageContent,
            fallback_context: Option<String>,
        ) -> Result<DiscordMessageId, GatewayError>;
    }

    #[async_trait]
    impl DiscordChannelGateway for DiscordGateway {
        async fn create_channel(
            &self,
            guild_id: DiscordGuildId,
            params: ChannelCreateParams,
        ) -> Result<DiscordChannelId, GatewayError>;

        async fn edit_channel(
            &self,
            channel_id: DiscordChannelId,
            params: ChannelEditParams,
        ) -> Result<(), GatewayError>;

        async fn delete_channel(
            &self,
            channel_id: DiscordChannelId,
        ) -> Result<(), GatewayError>;

        async fn get_channel(
            &self,
            channel_id: DiscordChannelId,
        ) -> Result<ChannelData, GatewayError>;
    }

    #[async_trait]
    impl DiscordInteractionGateway for DiscordGateway {
        async fn defer_interaction(
            &self,
            interaction_id: DiscordInteractionId,
            interaction_token: &str,
        ) -> Result<(), GatewayError>;

        async fn respond_to_interaction(
            &self,
            interaction_id: DiscordInteractionId,
            interaction_token: &str,
            response: InteractionResponse,
        ) -> Result<(), GatewayError>;

        async fn edit_interaction_response(
            &self,
            interaction_id: DiscordInteractionId,
            interaction_token: &str,
            response: InteractionResponse,
        ) -> Result<(), GatewayError>;
    }

    #[async_trait]
    impl DiscordReactionGateway for DiscordGateway {
        async fn get_reaction_users(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
            emoji: ReactionEmoji,
            limit: Option<u8>,
        ) -> Result<Vec<DiscordUserId>, GatewayError>;

        async fn add_reaction(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
            emoji: ReactionEmoji,
        ) -> Result<(), GatewayError>;

        async fn remove_own_reaction(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
            emoji: ReactionEmoji,
        ) -> Result<(), GatewayError>;
    }

    #[async_trait]
    impl DiscordGuildGateway for DiscordGateway {
        async fn get_member(
            &self,
            guild_id: DiscordGuildId,
            user_id: DiscordUserId,
        ) -> Result<GuildMember, GatewayError>;

        async fn get_roles(
            &self,
            guild_id: DiscordGuildId,
        ) -> Result<Vec<GuildRole>, GatewayError>;

        async fn get_emojis(
            &self,
            guild_id: DiscordGuildId,
        ) -> Result<Vec<GuildEmoji>, GatewayError>;
    }
}

/// テスト用ヘルパー：デフォルトのMessageDataを作成する
#[cfg(test)]
pub fn create_test_message_data(
    id: u64,
    channel_id: u64,
    author_id: u64,
    content: &str,
) -> MessageData {
    MessageData {
        id: DiscordMessageId::new(id),
        channel_id: DiscordChannelId::new(channel_id),
        author_id: DiscordUserId::new(author_id),
        content: content.to_string(),
        embeds: vec![],
        components: vec![],
        pinned: false,
    }
}

/// テスト用ヘルパー：デフォルトのGuildMemberを作成する
#[cfg(test)]
pub fn create_test_guild_member(user_id: u64, guild_id: u64, roles: Vec<u64>) -> GuildMember {
    use crate::types::discord::DiscordRoleId;

    GuildMember {
        user_id: DiscordUserId::new(user_id),
        guild_id: DiscordGuildId::new(guild_id),
        nickname: None,
        roles: roles.into_iter().map(DiscordRoleId::new).collect(),
        joined_at: None,
        premium_since: None,
        deaf: false,
        mute: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mock_gateway_send_message() {
        let mut mock = MockDiscordGateway::new();

        mock.expect_send_message()
            .returning(|_, _| Ok(DiscordMessageId::new(12345)));

        let result = mock
            .send_message(DiscordChannelId::new(111), MessageContent::text("Hello"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), 12345);
    }

    #[tokio::test]
    async fn test_mock_gateway_error_case() {
        let mut mock = MockDiscordGateway::new();

        mock.expect_send_message()
            .returning(|_, _| Err(GatewayError::SendMessageFailed("Test error".to_string())));

        let result = mock
            .send_message(DiscordChannelId::new(111), MessageContent::text("Hello"))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_gateway_get_reaction_users() {
        let mut mock = MockDiscordGateway::new();

        mock.expect_get_reaction_users().returning(|_, _, _, _| {
            Ok(vec![
                DiscordUserId::new(1001),
                DiscordUserId::new(1002),
                DiscordUserId::new(1003),
            ])
        });

        let result = mock
            .get_reaction_users(
                DiscordChannelId::new(111),
                DiscordMessageId::new(222),
                ReactionEmoji::unicode("👍"),
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_mock_gateway_arc_usage() {
        let mut mock = MockDiscordGateway::new();

        mock.expect_send_message()
            .returning(|_, _| Ok(DiscordMessageId::new(99999)));

        let gateway: Arc<dyn DiscordMessageGateway> = Arc::new(mock);

        let result = gateway
            .send_message(DiscordChannelId::new(111), MessageContent::text("Test"))
            .await;

        assert!(result.is_ok());
    }
}
