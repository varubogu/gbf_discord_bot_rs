use poise::serenity_prelude::AutocompleteChoice;
use std::sync::Arc;
use tracing::error;

use crate::events::converters::to_autocomplete_choices;
use crate::events::permission::check_bot_control_role;
use crate::facades::channel::ChannelManagementFacade;
use crate::types::{PoiseContext, Result};

/// チャンネル種別の選択肢を取得
async fn channel_type_autocomplete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    let facade = ChannelManagementFacade::new(Arc::new(ctx.data().app_state.clone()));
    match facade.get_channel_types_for_autocomplete().await {
        Ok(options) => to_autocomplete_choices(options),
        Err(e) => {
            error!(error = %e, "チャンネル種別の取得に失敗しました");
            vec![]
        }
    }
}

/// チャンネルを削除
///
/// ギルドの通知チャンネル設定を削除します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "channel_unregister",
    name_localized("ja", "チャンネル登録解除"),
    description_localized(
        "ja",
        "ギルドの通知チャンネル設定を削除します。（gbf_bot_controlロール必須）"
    )
)]
pub async fn channel_unregister(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_type_autocomplete"]
    #[name_localized("ja", "チャンネル種別")]
    #[description = "Channel type"]
    #[description_localized("ja", "削除するチャンネル種別")]
    channel_type: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    // channel_typeをi32に変換
    let channel_type_id: i32 =
        channel_type
            .parse()
            .map_err(|_| crate::types::AppError::Validation {
                field: "チャンネル種別".to_string(),
            })?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = ChannelManagementFacade::new(Arc::new(app_state.clone()));
    let result = facade
        .unregister_channel(guild_id.get() as i64, channel_type_id)
        .await?;

    // 結果メッセージを作成
    let mut message = format!(
        "✅ チャンネル設定を削除しました。\n\n**種別:** {}\n**削除されたチャンネル:** <#{}>\n\n**現在の設定状況:**\n",
        result.channel_type_name, result.old_channel_id
    );

    // ChannelDisplayServiceから取得した設定状況を整形
    for setting in &result.settings_display.settings {
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
