use log::info;
use poise::serenity_prelude::Reaction;
use crate::facades::battle_recruitment::member_update;

pub async fn on_reaction_remove(
    ctx: &poise::serenity_prelude::Context,
    reaction: &Reaction,
) -> Result<(), String> {
    // Create a reaction handler service
    info!("Reaction removed:");
    member_update(ctx).await;
    Ok(())
}