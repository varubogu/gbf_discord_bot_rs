use log::info;
use poise::serenity_prelude::{Context, Reaction};

pub async fn on_reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), String> {
    info!("Reaction added:");

    // Extract required IDs from reaction
    let guild_id = reaction.guild_id.map(|id| id.get()).unwrap_or(0);
    let channel_id = reaction.channel_id.get();
    let message_id = reaction.message_id.get();

    // Note: This function needs AppState to create facade, but it's not available in this context
    // This is a temporary placeholder implementation
    info!(
        "Would update participants for guild: {}, channel: {}, message: {}",
        guild_id, channel_id, message_id
    );

    // TODO: Implement proper facade call when AppState is available
    Ok(())
}
