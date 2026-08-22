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
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
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

/// テスト用データベースの複製元となるテンプレートDB名
const TEMPLATE_DB_NAME: &str = "gbf_test_template";
/// アプリ用DBロールのテスト時パスワード
const TEST_ROLE_PASSWORD: &str = "test_password";

/// ファサード結合テストで共有するPostgreSQLコンテナ
struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    port: u16,
}

static TEST_DATABASE: OnceCell<TestDatabase> = OnceCell::const_new();

/// 払い出し済みテストDBの連番（同一テストバイナリ内で一意）
static DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

async fn test_database() -> &'static TestDatabase {
    TEST_DATABASE
        .get_or_init(|| async {
            // テストごとにDBを分ける都合で接続数が増えるため、上限を既定の100から引き上げる。
            // `-c fsync=off` は testcontainers-modules の既定値だが、
            // cmdを上書きすると失われるためここで明示する。
            let container = Postgres::default()
                .with_cmd(["-c", "fsync=off", "-c", "max_connections=200"])
                .start()
                .await
                .unwrap_or_else(|error| panic!("テスト用PostgreSQLの起動に失敗しました: {error}"));
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .unwrap_or_else(|error| {
                    panic!("テスト用PostgreSQLのポート取得に失敗しました: {error}")
                });

            let admin_db = admin_connection(port).await;
            create_test_roles(&admin_db).await;
            create_template_database(&admin_db, port).await;
            close_connection(admin_db).await;

            TestDatabase {
                _container: container,
                port,
            }
        })
        .await
}

/// テンプレートDBを作成し、マイグレーションとマスターデータ投入まで済ませる。
///
/// テンプレートDBへの接続が残っていると `CREATE DATABASE ... TEMPLATE` が失敗するため、
/// 初期化に使った接続はこの関数を抜ける前に必ずクローズする。
async fn create_template_database(admin_db: &DatabaseConnection, port: u16) {
    admin_db
        .execute_unprepared(&format!("CREATE DATABASE {TEMPLATE_DB_NAME}"))
        .await
        .unwrap_or_else(|error| panic!("テンプレートDBの作成に失敗しました: {error}"));

    let template_db = connect("postgres", "postgres", TEMPLATE_DB_NAME, port).await;
    template_db
        .execute_unprepared("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .unwrap_or_else(|error| {
            panic!("テスト用PostgreSQLのpgcrypto拡張有効化に失敗しました: {error}")
        });
    Migrator::up(&template_db, None)
        .await
        .unwrap_or_else(|error| {
            panic!("テスト用データベースのマイグレーションに失敗しました: {error}")
        });
    seed_master_data(&template_db).await;

    close_connection(template_db).await;
}

/// 管理用の一時接続を取得する
///
/// `#[tokio::test]` はテストごとにランタイムを作って終了時に破棄するため、
/// 接続プールを `static` に保持すると最初のテストの終了と同時に使えなくなる。
/// そのため管理接続は必要になるたびに張り、使い終わったらクローズする。
async fn admin_connection(port: u16) -> DatabaseConnection {
    connect("postgres", "postgres", "postgres", port).await
}

/// 一時接続をクローズする
async fn close_connection(connection: DatabaseConnection) {
    connection
        .close()
        .await
        .unwrap_or_else(|error| panic!("テスト用DB接続のクローズに失敗しました: {error}"));
}

fn database_url(user: &str, password: &str, db_name: &str, port: u16) -> String {
    format!("postgres://{user}:{password}@127.0.0.1:{port}/{db_name}")
}

/// テスト用DBへ接続する
///
/// テストごとにDBを分けるとプールの数がテスト数分だけ増えるため、
/// 1プールあたりの接続数を絞り、アイドル接続を早めに返却する。
async fn connect(user: &str, password: &str, db_name: &str, port: u16) -> DatabaseConnection {
    let mut options = ConnectOptions::new(database_url(user, password, db_name, port));
    options
        .max_connections(4)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(5));

    Database::connect(options).await.unwrap_or_else(|error| {
        panic!("テスト用DB {db_name} への{user}ロール接続に失敗しました: {error}")
    })
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
                "CREATE ROLE {role} LOGIN PASSWORD '{TEST_ROLE_PASSWORD}'"
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

/// テスト1件ごとに払い出される独立したデータベース
///
/// テンプレートDBの複製として作られるため、テスト間でデータが干渉しない。
/// `create_test_app_state` は呼び出しのたびに新しいDBを作るので、
/// 1つのテスト内で複数の接続を同一DBへ張りたい場合はこのハンドルを経由する。
pub struct TestDb {
    name: String,
    port: u16,
}

impl TestDb {
    /// テンプレートDBを複製して、このテスト専用のデータベースを払い出す
    pub async fn new() -> Self {
        let database = test_database().await;
        let name = format!(
            "gbf_test_{}",
            DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );

        // テンプレートDBには初期化後どのセッションも接続しないため、
        // 複数テストから同時に複製しても問題ない（直列化すると律速要因になる）。
        let admin_db = admin_connection(database.port).await;
        admin_db
            .execute_unprepared(&format!(
                "CREATE DATABASE {name} TEMPLATE {TEMPLATE_DB_NAME}"
            ))
            .await
            .unwrap_or_else(|error| panic!("テスト用DB {name} の作成に失敗しました: {error}"));
        close_connection(admin_db).await;

        Self {
            name,
            port: database.port,
        }
    }

    /// このデータベースに紐づくテスト用のAppStateを作成する
    ///
    /// 本番同様に Guild/System/Global の3接続を個別に張る。
    /// ただし検証用クエリがRLSセッション変数に依存しないよう、
    /// 結合テストではいずれも管理者ロールで接続する。
    /// RLS固有の振る舞いは `guild_role_db` を使う専用テストで検証する。
    pub async fn app_state(&self) -> AppState {
        let config = AppConfig {
            discord_token: "test_token".to_string(),
            db_host: "127.0.0.1".to_string(),
            db_port: self.port,
            db_name: self.name.clone(),
            max_schedule_days_outside_event: 365,
        };

        AppState::new(
            self.guild_db().await,
            self.guild_db().await,
            self.guild_db().await,
            config,
        )
    }

    /// データ準備・検証用の直接接続を取得する
    pub async fn guild_db(&self) -> DatabaseConnection {
        connect("postgres", "postgres", &self.name, self.port).await
    }

    /// RLS検証用のGuildロール接続を取得する
    #[allow(dead_code)]
    pub async fn guild_role_db(&self) -> DatabaseConnection {
        connect("gbf_bot_guild", TEST_ROLE_PASSWORD, &self.name, self.port).await
    }

    /// 管理者ロールが必要な操作用の接続を取得する
    #[allow(dead_code)]
    pub async fn admin_db(&self) -> DatabaseConnection {
        connect("gbf_bot_admin", TEST_ROLE_PASSWORD, &self.name, self.port).await
    }
}

/// テスト用のAppStateを作成
///
/// 呼び出しのたびにテスト専用のデータベースを新規作成する。
/// そのため1つのテスト内で2回呼ぶと互いに無関係な2つのDBができる。
/// AppState以外の接続も必要なテストでは `TestDb` を直接使うこと。
#[allow(dead_code)]
pub async fn create_test_app_state() -> AppState {
    TestDb::new().await.app_state().await
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
