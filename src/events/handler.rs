use crate::events::handlers;
use crate::types::{AppError, PoiseData, Result};
use tracing::{debug, info};

#[allow(dead_code)]
pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, PoiseData, AppError>,
    data: &PoiseData,
) -> Result<()> {
    match event {
        poise::serenity_prelude::FullEvent::Ready { data_about_bot } => {
            println!("Connected as {}", data_about_bot.user.name);
            handlers::ready::on_ready(ctx).await?;
        }
        poise::serenity_prelude::FullEvent::GuildCreate { guild, .. } => {
            handlers::guild_create::on_guild_create(ctx, guild, data).await?;
        }
        poise::serenity_prelude::FullEvent::ReactionAdd { add_reaction } => {
            println!(
                "reaction add user is {}",
                add_reaction.user(&ctx.http).await?.name
            );
            handlers::reaction_add::on_reaction_add(ctx, add_reaction, data).await?;
        }
        poise::serenity_prelude::FullEvent::ReactionRemove { removed_reaction } => {
            println!(
                "reaction removes user is {}",
                removed_reaction.user(&ctx.http).await?.name
            );
            handlers::reaction_remove::on_reaction_remove(ctx, removed_reaction, data).await?;
        }
        poise::serenity_prelude::FullEvent::InteractionCreate { interaction } => {
            debug!(
                interaction_type = ?interaction.kind(),
                "InteractionCreateイベントを受信"
            );

            // ComponentInteraction（ボタンクリック等）を処理
            if let Some(component_interaction) = interaction.as_message_component() {
                info!(
                    custom_id = %component_interaction.data.custom_id,
                    "ComponentInteractionを検出"
                );
                handlers::component_interaction::on_component_interaction(
                    ctx,
                    component_interaction,
                    data,
                )
                .await?;
            }
            // ModalInteraction（モーダル送信）を処理
            else if let Some(modal_interaction) = interaction.as_modal_submit() {
                // 日時入力モーダルの処理
                if modal_interaction
                    .data
                    .custom_id
                    .starts_with("recruit_change_date_modal:")
                {
                    use crate::events::interactions::modal::recruit_change_date_modal;
                    recruit_change_date_modal::handle_recruit_change_date_modal(
                        ctx,
                        modal_interaction,
                        data,
                    )
                    .await?;
                }
            }
        }
        poise::serenity_prelude::FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            handlers::message_delete::on_message_delete(
                ctx,
                *channel_id,
                *deleted_message_id,
                *guild_id,
                data,
            )
            .await?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
