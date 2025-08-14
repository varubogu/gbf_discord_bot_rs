use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

mod events;
mod services;
mod utils;
mod models;
mod types;
mod repository;
mod facades;

use crate::events::handler::event_handler;
use crate::types::{PoiseData, PoiseError};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env");
    dotenv::from_path(dotenv_path).ok();

    // Get Discord token
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Set up intents
    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create poise framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands(),
            ..Default::default()
        })
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let db_conn = db_connect().await?;
                let database_service: Arc<dyn crate::utils::database::DatabaseService> = Arc::new(
                    crate::utils::database::SeaOrmDatabase::new(db_conn)
                );

                // PoiseDataを初期化
                let data = PoiseData {
                    db: database_service,
                };

                Ok(data)
            })
        })
        .build();

    // Create client with poise
    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn db_connect() -> Result<DatabaseConnection, DbErr> {
    // データベースURL取得
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // SeaORMでの接続設定
    let mut opt = ConnectOptions::new(&database_url);
    opt.max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    // SeaORMでDB接続
    let db: DatabaseConnection = Database::connect(opt).await?;
    Ok(db)
}

fn commands() -> Vec<poise::Command<PoiseData, PoiseError>> {
    vec![
        // events::interactions::command_interactions::slash::environ_load::handle_environ_load_command(),
        // events::interactions::command_interactions::contextmenu::reaction_users_context_menu::get_reaction_members(),
        // events::interactions::command_interactions::contextmenu::reaction_grouping_users_context_menu::get_reaction_grouping_members(),
    ]
}

#[allow(dead_code)]
async fn error_handler(error: poise::FrameworkError<'_, PoiseData, PoiseError>) {
    println!("Oh no, we got an error: {:?}", error);
}