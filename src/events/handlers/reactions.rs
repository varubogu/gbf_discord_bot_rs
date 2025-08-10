use std::sync::Arc;

use serenity::all::{Context, Reaction};

use crate::services::database::Database;
use crate::services::battle::ReactionHandler;

pub async fn handle_reaction_add(
    ctx: Context,
    reaction: Reaction,
    db: Arc<Database>,
) -> Result<(), String> {
    // Create a reaction handler service
    let handler = ReactionHandler::new(db);
    
    // Delegate to the service
    handler.handle_reaction_add(ctx, reaction).await
}

pub async fn handle_reaction_remove(
    ctx: Context,
    reaction: Reaction,
    db: Arc<Database>,
) -> Result<(), String> {
    // Create a reaction handler service
    let handler = ReactionHandler::new(db);
    
    // Delegate to the service
    handler.handle_reaction_remove(ctx, reaction).await
}