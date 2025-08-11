use poise::serenity_prelude::Reaction;
use crate::services::battle::ReactionHandler;

pub async fn on_ready(
    ctx: &poise::serenity_prelude::Context,
) -> Result<(), String> {
    println!("on_ready");
    Ok(())
}
