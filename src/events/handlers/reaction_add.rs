use log::info;
use poise::serenity_prelude::Reaction;
use crate::facades::battle_recruitment::member_update;

pub async fn on_reaction_add(
    ctx: &poise::serenity_prelude::Context,
    reaction: &Reaction,
) -> Result<(), String> {
    info!("Reaction added:");
    member_update(ctx).await;
    Ok(())
}