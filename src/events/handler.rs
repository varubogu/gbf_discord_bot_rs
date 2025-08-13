
use crate::events::handlers;
use crate::types::{PoiseData, PoiseError};

pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, PoiseData, PoiseError>,
    data: &PoiseData,
) -> Result<(), PoiseError> {
    match event {
        poise::serenity_prelude::FullEvent::Ready { data_about_bot} => {
            println!("Connected as {}", data_about_bot.user.name);
            handlers::ready::on_ready(ctx).await?;
        },
        poise::serenity_prelude::FullEvent::ReactionAdd { add_reaction} => {
            println!("reaction add user is {}", add_reaction.user(&ctx.http).await?.name);
            handlers::reaction_add::on_reaction_add(ctx, add_reaction).await?;
        },
        poise::serenity_prelude::FullEvent::ReactionRemove { removed_reaction} => {
            println!("reaction removes user is {}", removed_reaction.user(&ctx.http).await?.name);
            handlers::reaction_remove::on_reaction_remove(ctx, removed_reaction).await?;
        },
        _ => {
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    

}