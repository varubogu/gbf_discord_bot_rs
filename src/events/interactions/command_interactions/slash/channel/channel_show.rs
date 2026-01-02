use std::sync::Arc;

use crate::facades::channel::ChannelManagementFacade;
use crate::types::{PoiseContext, Result};

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
        ctx.send(
            poise::CreateReply::default()
                .content("⚠️ チャンネル種別が登録されていません。")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // 設定状況を作成
    let mut message = "**現在のチャンネル設定:**\n\n".to_string();

    for setting in &settings_display.settings {
        if let Some(channel_id) = setting.channel_id {
            message.push_str(&format!(
                "• **{}**: <#{}>\n",
                setting.channel_type_name, channel_id
            ));
        } else {
            message.push_str(&format!("• **{}**: 未設定\n", setting.channel_type_name));
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
