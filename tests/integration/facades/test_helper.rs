// ファサード結合テスト用共通ヘルパー

use async_trait::async_trait;
use gbf_discord_bot_rs::errors::GatewayError;
use gbf_discord_bot_rs::gateway::{
    DiscordChannelGateway, DiscordGuildGateway, DiscordInteractionGateway, DiscordMessageGateway,
    DiscordReactionGateway,
};
use gbf_discord_bot_rs::types::discord::{
    ChannelCreateParams, ChannelData, ChannelEditParams, DiscordChannelId, DiscordGuildId,
    DiscordInteractionId, DiscordMessageId, DiscordUserId, GuildEmoji, GuildMember, GuildRole,
    InteractionResponse, MessageContent, MessageData, ReactionEmoji,
};
use gbf_discord_bot_rs::types::{AppConfig, AppState};
use migration::{Migrator, MigratorTrait};
use mockall::mock;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;

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

/// ファサード結合テストで共有するPostgreSQLコンテナ
struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    port: u16,
}

static TEST_DATABASE: OnceCell<TestDatabase> = OnceCell::const_new();

async fn test_database() -> &'static TestDatabase {
    TEST_DATABASE
        .get_or_init(|| async {
            let container = Postgres::default()
                .start()
                .await
                .unwrap_or_else(|error| panic!("テスト用PostgreSQLの起動に失敗しました: {error}"));
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .unwrap_or_else(|error| {
                    panic!("テスト用PostgreSQLのポート取得に失敗しました: {error}")
                });
            let admin_db = Database::connect(database_url("postgres", "postgres", port))
                .await
                .unwrap_or_else(|error| {
                    panic!("テスト用PostgreSQLへの管理者接続に失敗しました: {error}")
                });

            create_test_roles(&admin_db).await;
            admin_db
                .execute_unprepared("CREATE EXTENSION IF NOT EXISTS pgcrypto")
                .await
                .unwrap_or_else(|error| {
                    panic!("テスト用PostgreSQLのpgcrypto拡張有効化に失敗しました: {error}")
                });
            Migrator::up(&admin_db, None).await.unwrap_or_else(|error| {
                panic!("テスト用データベースのマイグレーションに失敗しました: {error}")
            });
            seed_master_data(&admin_db).await;

            TestDatabase {
                _container: container,
                port,
            }
        })
        .await
}

fn database_url(user: &str, password: &str, port: u16) -> String {
    format!("postgres://{user}:{password}@127.0.0.1:{port}/postgres")
}

async fn create_test_roles(admin_db: &DatabaseConnection) {
    for role in [
        "gbf_bot_system",
        "gbf_bot_guild",
        "gbf_bot_global",
        "gbf_bot_admin",
    ] {
        admin_db
            .execute_unprepared(&format!(
                "CREATE ROLE {role} LOGIN PASSWORD 'test_password'"
            ))
            .await
            .unwrap_or_else(|error| panic!("テスト用ロール {role} の作成に失敗しました: {error}"));
    }
}

/// 結合テストで共通利用する最小限のマスターデータを投入する。
async fn seed_master_data(admin_db: &DatabaseConnection) {
    const SEED_SQL: &str = r#"
        INSERT INTO master.channel_types (id, name, memo) VALUES
            (1, 'イベント通知', NULL),
            (2, 'マルチ募集', NULL),
            (3, '団連絡', NULL),
            (4, '共用マルチ募集', NULL),
            (5, '管理者通知', NULL)
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO master.battle_styles (id, display_name, reactions, sort_order) VALUES
            (1, '通常', '✅', 1),
            (2, '6属性', '🔥,💧,🌍,💨,☀️,🌑', 2)
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO master.elements (id, reaction_stamp, name_jp, name_en) VALUES
            (1, '🔥', '火', 'fire'),
            (2, '💧', '水', 'water'),
            (3, '🌍', '土', 'earth'),
            (4, '💨', '風', 'wind'),
            (5, '☀️', '光', 'light'),
            (6, '🌑', '闇', 'dark')
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO master.quests
            (id, name, default_battle_style_id, recruit_count, available_battle_style_ids, sort_order)
        VALUES
            (1, 'アルバハHL', 1, 6, '1,2', 1),
            (2, 'ルシファーHL', 1, 6, '1,2', 2),
            (3, 'ベルゼバブHL', 1, 6, '1,2', 3),
            (4, 'スーパーアルティメットバハムート', 2, 6, '1,2', 4)
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO master.quest_aliases (quest_id, sequence_no, alias, alias_kana_small)
        VALUES
            (1, 1, 'アルバハHL', 'あるばはhl'),
            (2, 1, 'ルシファーHL', 'るしふぁーhl'),
            (3, 1, 'ベルゼバブHL', 'べるぜばぶhl'),
            (4, 1, 'スーパーアルティメットバハムート', 'すーぱーあるてぃめっとばはむーと')
        ON CONFLICT (quest_id, sequence_no) DO NOTHING;

        SELECT setval('master.quests_id_seq', 4, true);
    "#;

    admin_db
        .execute_unprepared(SEED_SQL)
        .await
        .unwrap_or_else(|error| panic!("テスト用マスターデータの投入に失敗しました: {error}"));
}

async fn get_test_db(user: &str) -> DatabaseConnection {
    let database = test_database().await;
    let password = if user == "postgres" {
        "postgres"
    } else {
        "test_password"
    };
    Database::connect(database_url(user, password, database.port))
        .await
        .unwrap_or_else(|error| panic!("テスト用{user}ロールの接続に失敗しました: {error}"))
}

/// テスト用のGuildロール接続を取得
pub async fn get_test_guild_db() -> DatabaseConnection {
    get_test_db("postgres").await
}

/// RLS検証用のGuildロール接続を取得
pub async fn get_test_guild_role_db() -> DatabaseConnection {
    get_test_db("gbf_bot_guild").await
}

/// テスト用のAdminロール接続を取得
pub async fn get_test_admin_db() -> DatabaseConnection {
    get_test_db("gbf_bot_admin").await
}

/// テスト用のAppStateを作成
///
/// 3つのDBロール（Guild/System/Global）を本番同様に個別接続で初期化する。
/// テストでも既定のDB_USER/DB_PASSWORDには依存しない。
pub async fn create_test_app_state() -> AppState {
    let config = AppConfig {
        discord_token: "test_token".to_string(),
        db_host: "127.0.0.1".to_string(),
        db_port: test_database().await.port,
        db_name: "postgres".to_string(),
        max_schedule_days_outside_event: 365,
    };

    // 検証用クエリがRLSセッション変数に依存しないよう、結合テストでは管理者接続を共有する。
    // RLS固有の振る舞いはロール接続を使う専用テストで検証する。
    let guild_db = get_test_db("postgres").await;
    let system_db = get_test_db("postgres").await;
    let global_db = get_test_db("postgres").await;

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
