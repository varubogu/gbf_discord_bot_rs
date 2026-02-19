use std::sync::Arc;

use crate::events::helpers::get_message_or_fallback_from_context;
use crate::facades::channel::ChannelManagementFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;

/// チャンネル設定を表示
///
/// ギルドの通知チャンネル設定を表示します。
#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    rename = "channel_show",
    name_localized("ja", "チャンネル設定表示"),
    description_localized("ja", "ギルドの通知チャンネル設定を表示します。")
)]
pub async fn channel_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = ChannelManagementFacade::new(Arc::new(app_state.clone()));
    let settings_display = facade.show_channel_settings(guild_id.get() as i64).await?;

    // チャンネル種別が登録されていない場合
    if settings_display.settings.is_empty() {
        let message = get_message_or_fallback_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::ChannelShowEmpty,
            HashMap::new(),
            "⚠️ チャンネル種別が登録されていません。",
        )
        .await;
        ctx.send(
            poise::CreateReply::default()
                .content(message)
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let header = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ChannelShowHeader,
        HashMap::new(),
        "**現在のチャンネル設定:**\n\n",
    )
    .await;
    let unset = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::ChannelShowUnset,
        HashMap::new(),
        "未設定",
    )
    .await;

    // 設定状況を作成
    let mut message = header;

    for setting in &settings_display.settings {
        if let Some(channel_id) = setting.channel_id {
            message.push_str(&format!(
                "• **{}**: <#{}>\n",
                setting.channel_type_name, channel_id
            ));
        } else {
            message.push_str(&format!("• **{}**: {unset}\n", setting.channel_type_name));
        }
    }

    ctx.send(
        poise::CreateReply::default()
            .content(message)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
