pub mod constants;
mod errors;
mod events;
mod facades;
mod infrastructure;
mod models;
mod repository;
mod services;
mod types;
mod utils;

use crate::events::{command::commands, handler::event_handler};
use crate::types::{AppConfig, AppError, AppState, PoiseData, Result};
use crate::utils::error_formatter::ErrorFormatter;
use crate::utils::startup_validator::StartupValidator;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;
use std::env;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info};

async fn initialize_database(database_url: &str) -> Result<sea_orm::DatabaseConnection> {
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

    // マイグレーションの実行
    info!("Running database migrations...");
    Migrator::up(&db, None).await.map_err(|e| {
        error!("Migration failed: {:?}", e);
        AppError::Database(e)
    })?;
    info!("Database migrations completed successfully");

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
    let dotenv_path = Path::new(&config_folder).join(".env");
    dotenv::from_path(dotenv_path).ok();

    // Startup validation - check all required environment variables and files
    info!("Running startup validation...");
    match StartupValidator::validate_all().await {
        Ok(validator) => {
            validator.display_results();
            info!("✅ All startup validations passed");
        }
        Err(e) => {
            eprintln!("{}", e);
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

    // Initialise a database with an optimised connection pool
    let db_connection = initialize_database(&config.database_url)
        .await
        .map_err(|e| {
            // データベース接続エラーの場合、詳細な情報を表示
            if let AppError::Database(ref db_err) = e {
                let masked_url = ErrorFormatter::mask_database_url(&config.database_url);
                eprintln!("{}", ErrorFormatter::format_db_error(db_err, &masked_url));
            }
            e
        })?;
    info!("Database connection pool initialised successfully");

    // If migrate_only flag is set, exit after migrations
    if migrate_only {
        info!("Migration completed successfully, exiting");
        return Ok(());
    }

    // Create AppState
    let app_state = AppState::new(db_connection, config);
    info!("AppState initialized");

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
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

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

    // スケジュール通知タイマーをバックグラウンドで起動
    let app_state_for_scheduler = std::sync::Arc::new(app_state.clone());
    let http = client.http.clone();
    tokio::spawn(async move {
        use crate::events::handlers::schedule_handler::ScheduleNotificationTimer;
        let timer = std::sync::Arc::new(ScheduleNotificationTimer::new(
            app_state_for_scheduler,
            http,
        ));
        timer.start().await;
    });
    info!("スケジュール通知タイマーを起動しました");

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
            if let Err(e) = ctx.say(user_message).await {
                error!(error = %e, "エラーメッセージの送信に失敗しました");
            }
        }
        // その他のエラー
        other => {
            error!("Poise framework error: {:?}", other);
        }
    }
}
