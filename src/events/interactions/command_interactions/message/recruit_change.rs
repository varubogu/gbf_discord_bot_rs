use crate::events::helpers::get_message_from_context;
use crate::events::interactions::components::recruit_change_handler;
use crate::events::permission::resolve_bot_control;
use crate::facades::recruitment::change::check_can_change_recruitment;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::{
    CreateInteractionResponse, CreateInteractionResponseMessage, Message,
};
use std::collections::HashMap;

/// メッセージコンテキストメニューから募集内容変更を開始
#[poise::command(context_menu_command = "募集内容変更")]
pub async fn recruit_change_context_menu(ctx: PoiseContext<'_>, message: Message) -> Result<()> {
    // ApplicationContextかどうかを先に確認
    let poise::Context::Application(app_ctx) = ctx else {
        return Err(crate::types::AppError::Generic(
            "このコマンドはコンテキストメニューからのみ使用できます".to_string(),
        ));
    };

    // 実行者情報を解決（events層でDiscordコンテキストから取得）
    let invoker_user_id = ctx.author().id.get();
    let has_bot_control = resolve_bot_control(&ctx).await;

    let channel_id = message.channel_id.get();
    let message_id = message.id.get();
    let guild_id_opt = message
        .guild_id
        .map(|id| id.get())
        .or_else(|| ctx.guild_id().map(|id| id.get()));

    // 権限チェック: DBから募集作成者を取得して比較する
    // パネル表示前に確認し、権限なしの場合はエラーを即時返す
    if let Some(guild_id) = guild_id_opt {
        let check_result = check_can_change_recruitment(
            &ctx.data().app_state,
            guild_id,
            channel_id,
            message_id,
            invoker_user_id,
            has_bot_control,
        )
        .await;

        if let Err(AppError::Business { .. }) = check_result {
            let error_msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandChangePermissionDenied,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。"
                    .to_string()
            });

            app_ctx
                .interaction
                .create_response(
                    &ctx.serenity_context().http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(error_msg)
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }

        // NotFound などその他のエラーはスルーしてパネルを表示（後続の変更適用で対処）
    }

    let (content, components) = recruit_change_handler::build_panel_content_and_components(
        ctx.data(),
        invoker_user_id,
        channel_id,
        message_id,
        guild_id_opt,
    )
    .await?;

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

    Ok(())
}
