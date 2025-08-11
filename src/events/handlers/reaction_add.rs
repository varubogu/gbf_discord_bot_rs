use log::info;
use poise::serenity_prelude::Reaction;
use crate::services::battle::ReactionHandler;

pub async fn on_reaction_add(
    ctx: &poise::serenity_prelude::Context,
    reaction: &Reaction,
) -> Result<(), String> {
    // Create a reaction handler service
    let handler = ReactionHandler::new();

    // Delegate to the service
    // handler.handle_reaction_remove(ctx, reaction).await
    info!("Reaction added:");
    Ok(())
}