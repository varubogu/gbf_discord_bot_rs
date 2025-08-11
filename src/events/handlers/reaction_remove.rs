use std::sync::Arc;
use log::info;
use poise::serenity_prelude::Reaction;
use crate::services::battle::ReactionHandler;
use crate::services::database::Database;

pub async fn on_reaction_remove(
    ctx: &poise::serenity_prelude::Context,
    reaction: &Reaction,
) -> Result<(), String> {
    // Create a reaction handler service
    let handler = ReactionHandler::new();
    // Delegate to the service
    // handler.handle_reaction_remove(ctx, reaction).await
    info!("Reaction removed:");
    Ok(())
}