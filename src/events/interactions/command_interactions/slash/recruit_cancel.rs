use crate::facades::recruitment::cancel as CancelFacade;
use crate::types;
use crate::types::domain_interface_result::CanCancelResult;
use crate::types::{AppError, PoiseContext};
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, Http, Message,
};
use std::time::Duration;
use tracing::error;

#[poise::command(
    context_menu_command = "recruit_cancel",
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル")
)]
pub async fn cancel(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> types::Result<()> {
    ctx.defer().await?;

    // キャンセル可能か確認
    match CancelFacade::confirm_cancel(ctx, &message).await {
        Ok(_) => Ok(()), // 正常パターンと業務エラーを想定
        CanCancelResult::AlreadyCancelled(_) => ctx.say("エラーが発生しました。").await,

        Err(e) => {
            // システムエラーを想定
            error!("{:?}", e);
            let _ = ctx.say("エラーが発生しました。").await;
            // エラーの種類に関わらずBotは続行
            Ok(())
        }
    }
    // キャンセル実行
    match CancelFacade::execute_cancel(ctx, &message).await {
        Ok()
    }
}

async fn do_noting(
    ctx: PoiseContext<'_>,
    reply: ReplyHandle<'_>,
    interaction: &ComponentInteraction,
) -> types::Result<()> {
    // キャンセルをキャンセルされたら確認ボタン削除
    let reply_message = reply.into_message().await;
    match reply_message {
        Ok(msg) => msg.delete(ctx).await?,
        Err(_) => error!("a"),
    };
    Ok(
        match CancelFacade::send_result_response(
            ctx,
            interaction,
            "キャンセルを取り消しました。".to_string(),
        )
        .await
        {
            Ok(_) => (),
            Err(_) => (),
        },
    )
}

async fn cancel_execute(
    ctx: PoiseContext<'_>,
    message: Message,
    http: &&Http,
    interaction: &ComponentInteraction,
) -> types::Result<()> {
    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    match CancelFacade::cancel_recruitment(ctx, guild_id, channel_id, message_id).await {
        Ok(_) => {
            match CancelFacade::send_result_response(
                ctx,
                interaction,
                "募集がキャンセルされました。".to_string(),
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(_) => Ok(()),
            }
        }
        Err(e) => match CancelFacade::send_error_response(http, &interaction, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        },
    }
}
