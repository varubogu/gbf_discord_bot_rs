use crate::events::interactions::components::recruit_change_handler;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{
    CreateInteractionResponse, CreateInteractionResponseMessage, Message,
};

/// メッセージコンテキストメニューから募集内容変更を開始
#[poise::command(context_menu_command = "募集内容変更")]
pub async fn recruit_change_context_menu(ctx: PoiseContext<'_>, message: Message) -> Result<()> {
    let (content, components) = recruit_change_handler::build_panel_content_and_components(
        ctx.data(),
        ctx.author().id.get(),
        message.channel_id.get(),
        message.id.get(),
        message
            .guild_id
            .map(|id| id.get())
            .or_else(|| ctx.guild_id().map(|id| id.get())),
    )
    .await?;

    // ApplicationContextの場合のみ応答可能
    match ctx {
        poise::Context::Application(app_ctx) => {
            app_ctx
                .interaction
                .create_response(
                    &ctx.serenity_context().http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(content)
                            .components(components)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        _ => {
            return Err(crate::types::AppError::Generic(
                "このコマンドはコンテキストメニューからのみ使用できます".to_string(),
            ));
        }
    }

    Ok(())
}
