use log::info;
use poise::serenity_prelude::Reaction;
use crate::facades::battle_recruitment::member_update;

pub async fn on_reaction_add(
    ctx: &poise::serenity_prelude::Context,
    reaction: &Reaction,
) -> Result<(), String> {
    info!("Reaction added:");
    
    // Extract required IDs from reaction
    let guild_id = reaction.guild_id.map(|id| id.get()).unwrap_or(0);
    let channel_id = reaction.channel_id.get();
    let message_id = reaction.message_id.get();
    
    // Call member_update with the new signature
    match member_update(ctx, guild_id, channel_id, message_id).await {
        Ok(_) => {
            info!("Member update completed successfully");
            Ok(())
        },
        Err(e) => {
            info!("Member update failed: {}", e);
            Err(e)
        }
    }
}