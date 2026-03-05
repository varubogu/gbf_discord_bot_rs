use crate::events::helpers::get_message_from_context;
use crate::events::init_message::build_init_guide_message;
use crate::events::permission::check_bot_control_role;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use std::collections::HashMap;
use tracing::info;

/// 初期設定メッセージを表示
///
/// ギルド初期設定手順（スプレッドシート設定、通知先チャンネル設定）を案内する。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    rename = "init_message",
    name_localized("ja", "初期設定メッセージ"),
    description_localized("ja", "Botの初期設定手順メッセージを表示します")
)]
pub async fn init_message(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ErrorsGuildOnly,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| MessageTextId::ErrorsGuildOnly.as_str().to_string());
            return Err(AppError::Business { message });
        }
    };

    let content = build_init_guide_message(&ctx.data().app_state, guild_id).await;
    ctx.say(content).await?;

    info!(
        guild_id = guild_id,
        user_id = %ctx.author().id,
        "初期設定メッセージをコマンドで送信しました"
    );

    Ok(())
}
