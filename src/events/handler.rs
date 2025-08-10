use std::sync::Arc;

use serenity::all::{Context, EventHandler as SerenityEventHandler, Ready, Message, Reaction};
use serenity::async_trait;
use tracing::{info, error};

use crate::services::database::Database;
use crate::events::handlers::reactions::{handle_reaction_add, handle_reaction_remove};

pub struct EventHandler {
    db: Arc<Database>,
}

impl EventHandler {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SerenityEventHandler for EventHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        
        // Slash commands registration is handled elsewhere
    }
    
    async fn message(&self, _ctx: Context, _msg: Message) {
        // Handle messages if needed
    }
    
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if let Err(e) = handle_reaction_add(ctx, reaction, self.db.clone()).await {
            error!("Error handling reaction add: {}", e);
        }
    }
    
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        if let Err(e) = handle_reaction_remove(ctx, reaction, self.db.clone()).await {
            error!("Error handling reaction remove: {}", e);
        }
    }
}