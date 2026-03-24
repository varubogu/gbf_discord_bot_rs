// ファサード結合テスト用共通ヘルパー

use async_trait::async_trait;
use gbf_discord_bot_rs::errors::GatewayError;
use gbf_discord_bot_rs::gateway::{
    DiscordChannelGateway, DiscordGuildGateway, DiscordInteractionGateway, DiscordMessageGateway,
    DiscordReactionGateway,
};
use gbf_discord_bot_rs::types::DbRole;
use gbf_discord_bot_rs::types::discord::{
    ChannelCreateParams, ChannelData, ChannelEditParams, DiscordChannelId, DiscordGuildId,
    DiscordInteractionId, DiscordMessageId, DiscordUserId, GuildEmoji, GuildMember, GuildRole,
    InteractionResponse, MessageContent, MessageData, ReactionEmoji,
};
use gbf_discord_bot_rs::types::{AppConfig, AppState};
use mockall::mock;
use sea_orm::DatabaseConnection;

/// テスト用ギルドID（テストデータ用の固定値）
pub const TEST_GUILD_ID: i64 = 999999999;
/// テスト用チャンネルID
pub const TEST_CHANNEL_ID: i64 = 888888888;
/// テスト用メッセージID
pub const TEST_MESSAGE_ID: i64 = 777777777;
/// テスト用ユーザーID
pub const TEST_USER_ID: u64 = 666666666;

// テスト用MockDiscordGateway
//
// 結合テストでGateway依存のファサードをテストするためのモック。
// mockallのmock!マクロを使用して全Gatewayトレイトを実装する。
mock! {
    pub TestGateway {}

    #[async_trait]
    impl DiscordMessageGateway for TestGateway {
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
    impl DiscordChannelGateway for TestGateway {
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
    impl DiscordInteractionGateway for TestGateway {
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
    impl DiscordReactionGateway for TestGateway {
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
    impl DiscordGuildGateway for TestGateway {
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

/// テスト用のデータベース接続を取得
///
/// 指定ロールの環境変数からDB接続情報を取得し、DatabaseConnectionを返す。
/// テストでは既定のDB_USER/DB_PASSWORDにはフォールバックしない。
pub async fn get_test_db_for_role(role: DbRole) -> DatabaseConnection {
    gbf_discord_bot_rs::test_utils::connect_database_for_role(role)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "テスト用{}データベース接続に失敗しました: {}",
                role.description(),
                error
            )
        })
}

/// テスト用のGuildロール接続を取得
pub async fn get_test_guild_db() -> DatabaseConnection {
    get_test_db_for_role(DbRole::Guild).await
}

/// テスト用のAdminロール接続を取得
pub async fn get_test_admin_db() -> DatabaseConnection {
    get_test_db_for_role(DbRole::Admin).await
}

/// テスト用のAppStateを作成
///
/// 3つのDBロール（Guild/System/Global）を本番同様に個別接続で初期化する。
/// テストでも既定のDB_USER/DB_PASSWORDには依存しない。
pub async fn create_test_app_state() -> AppState {
    let config = AppConfig {
        discord_token: "test_token".to_string(),
        db_host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        db_port: std::env::var("DB_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432),
        db_name: std::env::var("DB_NAME").unwrap_or_else(|_| "test".to_string()),
        max_schedule_days_outside_event: 365,
    };

    let guild_db = get_test_db_for_role(DbRole::Guild).await;
    let system_db = get_test_db_for_role(DbRole::System).await;
    let global_db = get_test_db_for_role(DbRole::Global).await;

    AppState::new(guild_db, system_db, global_db, config)
}

/// テスト用のMessageDataを作成
#[allow(dead_code)]
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
        reactions: vec![],
        pinned: false,
        referenced_message_id: None,
    }
}

/// データベースが利用可能かチェックし、不可の場合はテストをスキップ
///
/// テスト関数の先頭で呼び出す。DBが利用できない場合はprintln!してreturnする。
#[macro_export]
macro_rules! skip_if_no_db {
    () => {
        let (available, missing) = gbf_discord_bot_rs::test_utils::check_database_availability();
        if !available {
            println!(
                "テストスキップ: データベース接続情報が不足しています: {:?}",
                missing
            );
            return;
        }
    };
}
