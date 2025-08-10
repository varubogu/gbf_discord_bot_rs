use std::env;
use std::path::Path;
use std::sync::Arc;

use serenity::all::{
    Client, GatewayIntents,
};
use tracing::{error, info};

// Import modules
mod events;
mod services;
mod utils;
mod models;

use services::database::Database;
use services::environment::init_environment;
use events::handler::EventHandler;

async fn initialize_bot() -> Result<Arc<Database>, Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    let db_arc = Arc::new(db);
    
    // Initialize environment with database
    if let Err(e) = init_environment(Some(db_arc.clone())).await {
        error!("Failed to initialize environment with database: {}", e);
        // Continue anyway, as we at least have the .env values
    }
    
    Ok(db_arc)
}

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

    // Initialize database and environment
    let db = match initialize_bot().await {
        Ok(db) => db,
        Err(e) => {
            error!("Error initializing bot: {:?}", e);
            return;
        }
    };

    // Create event handler
    let event_handler = EventHandler::new(db);

    // Create client
    let mut client = Client::builder(&token, intents)
        .event_handler(event_handler)
        .await
        .expect("Error creating client");

    // Set up error handling for the client
    info!("Starting client...");
    match client.start().await {
        Ok(_) => info!("Client exited successfully"),
        Err(e) => {
            error!("Client error: {:?}", e);
            // Log detailed error information
            match e {
                serenity::Error::Gateway(gateway_err) => {
                    error!("Gateway error: {:?}", gateway_err);
                },
                serenity::Error::Http(http_err) => {
                    error!("HTTP error: {:?}", http_err);
                },
                serenity::Error::Model(model_err) => {
                    error!("Model error: {:?}", model_err);
                },
                _ => {
                    error!("Other error: {:?}", e);
                }
            }
        }
    }
}