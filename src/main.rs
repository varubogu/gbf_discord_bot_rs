use std::env;
use std::path::Path;
use std::sync::Arc;
use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use tracing::{error, info};

// Import modules
mod events;
mod services;
mod utils;
mod models;

use services::database::Database;
use services::environment::init_environment;
use events::handler::EventHandler;

pub(crate) type PoiseData = ();
pub(crate) type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;

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
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    // Create event handler for non-command events
    let event_handler = EventHandler::new();

    // Create client with poise
    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .event_handler(event_handler)
        .await;

    client.unwrap().start().await.unwrap();
}
fn commands() -> Vec<poise::Command<PoiseData
    , PoiseError>> {
    vec![
        events::interactions::command_interactions::slash::reaction_members::reaction_members(),
        events::interactions::command_interactions::contextmenu::reaction_users_context_menu::get_reaction_members(),
        events::interactions::command_interactions::contextmenu::reaction_grouping_users_context_menu::get_reaction_grouping_members(),
    ]
}

#[allow(dead_code)]
async fn error_handler(error: poise::FrameworkError<'_, PoiseData, PoiseError>) {
    println!("Oh no, we got an error: {:?}", error);
}