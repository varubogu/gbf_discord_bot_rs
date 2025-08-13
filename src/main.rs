use std::env;
use std::path::Path;
use poise::FrameworkError::EventHandler;
use poise::serenity_prelude::{self as serenity, GatewayIntents};

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
                Ok(())
            })
        })
        .build();

    // Create client with poise
    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
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