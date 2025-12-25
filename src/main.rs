use gbf_discord_bot_rs::events::{
    command::{admin_commands, commands, global_commands},
    handler::event_handler,
};
use gbf_discord_bot_rs::repository::database::{
    battle_recruitments_repository::BattleRecruitmentsRepositoryImpl,
    recruitment_participants_repository::RecruitmentParticipantsRepositoryImpl,
    schedule::{
        ScheduledTaskDissolutionRepository, ScheduledTaskRepository,
    },
};
use gbf_discord_bot_rs::services::{message::MessageService, schedule::SchedulerManager};
use gbf_discord_bot_rs::types::{AppConfig, AppError, AppState, DbRole, PoiseData, Result};
use gbf_discord_bot_rs::utils::error_formatter::ErrorFormatter;
use gbf_discord_bot_rs::utils::startup_validator::StartupValidator;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

async fn initialize_database(
    database_url: &str,
    run_migration: bool,
) -> Result<sea_orm::DatabaseConnection> {
    info!("Initializing optimized database connection...");

    // SeaORMコネクションプールの最適化設定
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    let db = Database::connect(opt).await?;
    info!("Database connection pool initialised successfully");

    // マイグレーションの実行（必要な場合のみ）
    if run_migration {
        info!("Running database migrations...");
        Migrator::up(&db, None).await.map_err(|e| {
            error!("Migration failed: {:?}", e);
            AppError::Database(e)
        })?;
        info!("Database migrations completed successfully");
    }

    Ok(db)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with proper configuration
    // RUST_LOG環境変数でログレベルを制御可能
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
    info!("Starting Granblue Fantasy Discord Bot...");

    // Load environment variables
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env.app");
    dotenv::from_path(dotenv_path).ok();

    // Startup validation - check all required environment variables and files
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

    // Load configuration using a structured approach
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Check if we should only run migrations
    let args: Vec<String> = env::args().collect();
    let migrate_only = args.iter().any(|arg| arg == "migrate-only");

    // マイグレーション実行（Adminロールを使用）
    info!("Running database migrations with Admin role...");
    let admin_url = config.database_url(DbRole::Admin).map_err(|e| {
        error!("Admin DB接続設定の取得に失敗: {}", e);
        e
    })?;
    info!("Admin role: {}", DbRole::Admin.description());
    let _admin_db = initialize_database(&admin_url, true).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&admin_url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })?;

    // If migrate_only flag is set, exit after migrations
    if migrate_only {
        info!("Migration completed successfully, exiting");
        return Ok(());
    }

    // Initialise database connections for different roles (without migration)
    info!("Initializing database connections for all roles...");

    // Guild ロール（通常のコマンド実行用、RLS適用）
    let guild_url = config.database_url(DbRole::Guild).map_err(|e| {
        error!("Guild DB接続設定の取得に失敗: {}", e);
        e
    })?;
    info!("Guild role: {}", DbRole::Guild.description());
    let guild_db = initialize_database(&guild_url, false).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&guild_url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })?;

    // System ロール（スケジューラー用、RLS適用なし）
    let system_url = config.database_url(DbRole::System).map_err(|e| {
        error!("System DB接続設定の取得に失敗: {}", e);
        e
    })?;
    info!("System role: {}", DbRole::System.description());
    let system_db = initialize_database(&system_url, false).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&system_url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })?;

    // Global ロール（マスターデータ更新用、RLS適用なし）
    let global_url = config.database_url(DbRole::Global).map_err(|e| {
        error!("Global DB接続設定の取得に失敗: {}", e);
        e
    })?;
    info!("Global role: {}", DbRole::Global.description());
    let global_db = initialize_database(&global_url, false).await.map_err(|e| {
        if let AppError::Database(ref db_err) = e {
            let masked_url = ErrorFormatter::mask_database_url(&global_url);
            eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
        }
        e
    })?;

    info!("All database connection pools initialised successfully");

    // タイムゾーンキャッシュを初期化（プログラム起動時に計算）
    info!("Initializing timezone cache...");
    gbf_discord_bot_rs::services::timezone_service::TimezoneService::initialize_timezone_cache();

    // Create AppState with all DB connections
    let app_state = AppState::new(guild_db, system_db, global_db, config);
    info!("AppState initialized with all role connections");

    // Set up Discord intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create Discord client (clone discord_token before the move)
    let discord_token = app_state.config.discord_token.clone();

    // Create a poise framework with AppState
    let app_state_for_framework = app_state.clone();
    let framework = poise::Framework::builder()
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
                match env::var("BOT_ADMIN_SERVER_ID") {
                    Ok(admin_server_id) => {
                        match admin_server_id.parse::<u64>() {
                            Ok(guild_id_u64) => {
                                let guild_id = serenity::GuildId::new(guild_id_u64);
                                let admin_cmds = admin_commands();
                                poise::builtins::register_in_guild(ctx, &admin_cmds, guild_id)
                                    .await?;
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
                        }
                    }
                    Err(_) => {
                        error!(
                            "⚠️ BOT_ADMIN_SERVER_ID not set - admin commands will not be registered"
                        );
                    }
                }

                // PoiseDataにAppStateを設定
                let data = PoiseData {
                    app_state: app_state_for_framework,
                };
                info!("Poise framework initialized with AppState");

                Ok(data)
            })
        })
        .build();
    let mut client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .await?;

    info!("Discord client created, starting bot...");

    // SchedulerManagerの初期化と起動
    let task_repo = Arc::new(ScheduledTaskRepository::new());
    let dissolution_repo = Arc::new(ScheduledTaskDissolutionRepository::new());
    let recruitment_repo = Arc::new(BattleRecruitmentsRepositoryImpl::new());
    let participants_repo = Arc::new(RecruitmentParticipantsRepositoryImpl::new());
    let message_service = Arc::new(MessageService::new());

    let mut scheduler_manager = SchedulerManager::new(
        Arc::new(app_state.system_db().clone()),
        client.http.clone(),
        task_repo,
        dissolution_repo,
        recruitment_repo,
        participants_repo,
        message_service,
    )
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

    // Start the bot
    if let Err(e) = client.start().await {
        error!("Error starting bot: {:?}", e);
        return Err(AppError::Discord(e));
    }

    Ok(())
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
            let user_message = error.user_message();
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
