pub mod constants;
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
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Granblue Fantasy Discord Bot...");

    // Load environment variables
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env");
    dotenv::from_path(dotenv_path).ok();

    // Load configuration using a structured approach
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Initialise a database with an optimised connection pool
    let db_connection = initialize_database(&config.database_url).await?;
    info!("Database connection pool initialised successfully");

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
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // PoiseDataにAppStateを設定
                let data = PoiseData { app_state };
                info!("Poise framework initialized with AppState");

                Ok(data)
            })
        })
        .build();
    let mut client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .await?;

    info!("Discord client created, starting bot...");

    // Start the bot
    if let Err(e) = client.start().await {
        error!("Error starting bot: {:?}", e);
        return Err(AppError::Discord(e));
    }

    Ok(())
}

#[allow(dead_code)]
async fn error_handler(error: poise::FrameworkError<'_, PoiseData, AppError>) {
    error!("Poise framework error: {:?}", error);
}
