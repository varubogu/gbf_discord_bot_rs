use gbf_discord_bot_rs::events::{
    command::{admin_commands, commands, global_commands},
    handler::event_handler,
    helpers::resolve_guild_locale,
};
use gbf_discord_bot_rs::facades::schedule::SchedulerTaskDispatchFacade;
use gbf_discord_bot_rs::gateway::PoiseDiscordGateway;
use gbf_discord_bot_rs::services::message::MessageTextId;
use gbf_discord_bot_rs::services::schedule::{SchedulerManager, TaskDispatchService};
use gbf_discord_bot_rs::types::{AppConfig, AppError, AppState, DbRole, PoiseData, Result};
use gbf_discord_bot_rs::utils::error_formatter::ErrorFormatter;
use gbf_discord_bot_rs::utils::startup_validator::StartupValidator;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("Starting Granblue Fantasy Discord Bot... version: {version}");
    initialize_logging();
    info!("Starting Granblue Fantasy Discord Bot...");

    load_environment();
    let config = load_and_validate_config().await?;

    run_migrations(&config).await?;

    if is_migrate_only() {
        info!("Migration completed successfully, exiting");
        return Ok(());
    }

    let app_state = create_app_state(config).await?;

    // タイムゾーンキャッシュを初期化
    info!("Initializing timezone cache...");
    gbf_discord_bot_rs::services::timezone_service::initialize_timezone_cache();

    let mut client = create_discord_client(&app_state).await?;

    start_scheduler(&app_state, client.http.clone()).await?;

    info!("Starting bot...");
    if let Err(e) = client.start().await {
        error!("Error starting bot: {:?}", e);
        return Err(AppError::Discord(Box::new(e)));
    }

    Ok(())
}

/// ロギングシステムを初期化
/// RUST_LOG環境変数でログレベルを制御可能（デフォルト: info）
fn initialize_logging() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

/// 環境変数ファイルを読み込む
fn load_environment() {
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env.app");
    dotenv::from_path(dotenv_path).ok();
}

/// スタートアップ時の環境変数・ファイル検証を実行し、設定を読み込む
async fn load_and_validate_config() -> Result<AppConfig> {
    info!("Running startup validation...");
    match StartupValidator::validate_all().await {
        Ok(validator) => {
            validator.display_results();
            info!("✅ All startup validations passed");
        }
        Err(e) => {
            eprintln!("{e}");
            error!("❌ Startup validation failed, exiting");
            std::process::exit(1);
        }
    }

    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");
    Ok(config)
}

/// migrate-onlyオプションが指定されているか確認
fn is_migrate_only() -> bool {
    env::args().any(|arg| arg == "migrate-only")
}

/// マイグレーションを実行する
async fn run_migrations(config: &AppConfig) -> Result<()> {
    info!("Running database migrations with Admin role...");
    let admin_url = config.database_url(DbRole::Admin).map_err(|e| {
        error!("Admin DB接続設定の取得に失敗: {}", e);
        e
    })?;
    info!("Admin role: {}", DbRole::Admin.description());

    let admin_db = create_database_connection(&admin_url).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&admin_url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })?;

    // マイグレーション実行
    info!("Running database migrations...");
    Migrator::up(&admin_db, None).await.map_err(|e| {
        error!("Migration failed: {:?}", e);
        AppError::Database(e)
    })?;
    info!("Database migrations completed successfully");

    Ok(())
}

/// 各ロール用のDB接続を作成してAppStateを構築する
async fn create_app_state(config: AppConfig) -> Result<AppState> {
    info!("Initializing database connections for all roles...");

    // Guild ロール（通常のコマンド実行用、RLS適用）
    let guild_db = create_role_connection(&config, DbRole::Guild).await?;

    // System ロール（スケジューラー用、RLS適用なし）
    let system_db = create_role_connection(&config, DbRole::System).await?;

    // Global ロール（マスターデータ更新用、RLS適用なし）
    let global_db = create_role_connection(&config, DbRole::Global).await?;

    info!("All database connection pools initialised successfully");

    let app_state = AppState::new(guild_db, system_db, global_db, config);
    info!("AppState initialized with all role connections");

    Ok(app_state)
}

/// 指定されたロールのDB接続を作成する
async fn create_role_connection(
    config: &AppConfig,
    role: DbRole,
) -> Result<sea_orm::DatabaseConnection> {
    let url = config.database_url(role).map_err(|e| {
        error!("{} DB接続設定の取得に失敗: {}", role.description(), e);
        e
    })?;
    info!("{:?} role: {}", role, role.description());

    create_database_connection(&url).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })
}

/// DB接続プールを作成する
async fn create_database_connection(database_url: &str) -> Result<sea_orm::DatabaseConnection> {
    // SeaORMコネクションプールの最適化設定
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    info!("Database connection pool initialised successfully");

    Ok(db)
}

/// Discord用のGatewayIntentsを設定する
fn create_gateway_intents() -> GatewayIntents {
    GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT
}

/// Poiseフレームワークを構築する
fn build_framework(app_state: AppState) -> poise::Framework<PoiseData, AppError> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(error_handler(error)),
            ..Default::default()
        })
        .setup(move |ctx, _ready, _framework| {
            Box::pin(async move {
                // グローバルコマンドを全サーバーに登録
                let global_cmds = global_commands();
                poise::builtins::register_globally(ctx, &global_cmds).await?;
                info!("Registered {} global commands", global_cmds.len());

                // 管理サーバー専用コマンドを特定ギルドにのみ登録
                register_admin_commands(ctx).await?;

                // PoiseDataにAppStateを設定
                let data = PoiseData { app_state };
                info!("Poise framework initialized with AppState");

                Ok(data)
            })
        })
        .build()
}

/// 管理サーバー専用コマンドを登録する
async fn register_admin_commands(
    ctx: &serenity::Context,
) -> std::result::Result<(), serenity::Error> {
    match env::var("BOT_ADMIN_SERVER_ID") {
        Ok(admin_server_id) => match admin_server_id.parse::<u64>() {
            Ok(guild_id_u64) => {
                let guild_id = serenity::GuildId::new(guild_id_u64);
                let admin_cmds = admin_commands();
                poise::builtins::register_in_guild(ctx, &admin_cmds, guild_id).await?;
                info!(
                    "Registered {} admin commands in guild {} ({})",
                    admin_cmds.len(),
                    guild_id,
                    admin_server_id
                );
            }
            Err(e) => {
                error!(
                    "BOT_ADMIN_SERVER_ID '{}' is not a valid number: {}",
                    admin_server_id, e
                );
            }
        },
        Err(_) => {
            error!("⚠️ BOT_ADMIN_SERVER_ID not set - admin commands will not be registered");
        }
    }
    Ok(())
}

/// Discordクライアントを作成する
async fn create_discord_client(app_state: &AppState) -> Result<serenity::Client> {
    let discord_token = app_state.config.discord_token.clone();
    let intents = create_gateway_intents();
    let framework = build_framework(app_state.clone());

    let client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .await?;

    info!("Discord client created");
    Ok(client)
}

/// SchedulerManagerを初期化してバックグラウンドで起動する
async fn start_scheduler(app_state: &AppState, http: Arc<serenity::Http>) -> Result<()> {
    let repos = app_state.repositories;
    let recruitment_repo = Arc::new(repos.battle_recruitments);
    let participants_repo = Arc::new(repos.recruitment_participants);
    let message_service = app_state.message_service.clone();
    let task_dispatch_service =
        TaskDispatchService::new(recruitment_repo, participants_repo, message_service, repos);
    let dispatch_facade = Arc::new(SchedulerTaskDispatchFacade::new(
        app_state.system_db.clone(),
        task_dispatch_service,
    ));

    // poise依存をサービス層から分離するため、ここでGatewayを作成
    let gateway = Arc::new(PoiseDiscordGateway::new(http));

    let mut scheduler_manager = SchedulerManager::new(gateway, dispatch_facade)
        .await
        .map_err(|e| {
            error!(error = %e, "SchedulerManagerの初期化に失敗しました");
            e
        })?;

    // SchedulerManagerをバックグラウンドで起動
    tokio::spawn(async move {
        if let Err(e) = scheduler_manager.start().await {
            error!(error = %e, "SchedulerManagerの起動に失敗しました");
        }
    });
    info!("SchedulerManagerを起動しました");

    Ok(())
}

async fn resolve_user_error_message(
    ctx: &poise::Context<'_, PoiseData, AppError>,
    app_error: &AppError,
) -> String {
    let guild_id = ctx.guild_id().map(|id| id.get() as i64);
    let locale = resolve_guild_locale(&ctx.data().app_state, guild_id).await;
    let message_service = ctx.data().app_state.message_service();
    let db = ctx.data().app_state.guild_db();

    match app_error {
        AppError::Database(_) => message_service
            .get_message(
                db,
                MessageTextId::AppErrorDatabase.as_str(),
                HashMap::new(),
                guild_id,
                Some(&locale),
            )
            .await
            .unwrap_or_else(|_| app_error.user_message()),
        AppError::Discord(_) => message_service
            .get_message(
                db,
                MessageTextId::AppErrorDiscord.as_str(),
                HashMap::new(),
                guild_id,
                Some(&locale),
            )
            .await
            .unwrap_or_else(|_| app_error.user_message()),
        AppError::Config { message } => {
            let mut params = HashMap::new();
            params.insert("message".to_string(), message.clone());
            message_service
                .get_message(
                    db,
                    MessageTextId::AppErrorConfig.as_str(),
                    params,
                    guild_id,
                    Some(&locale),
                )
                .await
                .unwrap_or_else(|_| app_error.user_message())
        }
        AppError::Validation { field } => {
            let mut params = HashMap::new();
            params.insert("field".to_string(), field.clone());
            message_service
                .get_message(
                    db,
                    MessageTextId::AppErrorValidation.as_str(),
                    params,
                    guild_id,
                    Some(&locale),
                )
                .await
                .unwrap_or_else(|_| app_error.user_message())
        }
        AppError::DiscordOperation(err) => {
            let mut params = HashMap::new();
            params.insert("message".to_string(), err.to_string());
            message_service
                .get_message(
                    db,
                    MessageTextId::AppErrorDiscordOperation.as_str(),
                    params,
                    guild_id,
                    Some(&locale),
                )
                .await
                .unwrap_or_else(|_| app_error.user_message())
        }
        AppError::ChannelCreationFailed => message_service
            .get_message(
                db,
                MessageTextId::AppErrorChannelCreationFailed.as_str(),
                HashMap::new(),
                guild_id,
                Some(&locale),
            )
            .await
            .unwrap_or_else(|_| app_error.user_message()),
        AppError::InCategoryChannelError => message_service
            .get_message(
                db,
                MessageTextId::AppErrorInCategoryChannel.as_str(),
                HashMap::new(),
                guild_id,
                Some(&locale),
            )
            .await
            .unwrap_or_else(|_| app_error.user_message()),
        AppError::Business { .. } | AppError::Generic(_) | AppError::NotFound(_) => {
            app_error.user_message()
        }
    }
}

async fn error_handler(error: poise::FrameworkError<'_, PoiseData, AppError>) {
    use poise::FrameworkError;

    match error {
        // コマンド実行時のエラー
        FrameworkError::Command { error, ctx, .. } => {
            // ログには詳細なエラー情報を出力
            error!(
                error = %error,
                command = %ctx.command().name,
                user_id = %ctx.author().id,
                guild_id = ?ctx.guild_id(),
                "コマンド実行エラー"
            );

            // Discord上にはユーザーフレンドリーなメッセージのみ表示
            let user_message = resolve_user_error_message(&ctx, &error).await;
            if let Err(e) = ctx
                .send(
                    poise::CreateReply::default()
                        .content(user_message)
                        .ephemeral(true),
                )
                .await
            {
                error!(error = %e, "エラーメッセージの送信に失敗しました");
            }
        }
        // その他のエラー
        other => error!("Poise framework error: {:?}", other),
    }
}
