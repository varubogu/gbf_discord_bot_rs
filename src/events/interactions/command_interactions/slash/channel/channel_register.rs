use poise::serenity_prelude::{AutocompleteChoice, Channel};
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

/// チャンネルを登録
///
/// ギルドの通知チャンネルを登録します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "channel_register",
    name_localized("ja", "チャンネル登録"),
    description_localized(
        "ja",
        "ギルドの通知チャンネルを登録します。（gbf_bot_controlロール必須）"
    )
)]
pub async fn channel_register(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_type_autocomplete"]
    #[name_localized("ja", "チャンネル種別")]
    #[description = "Channel type"]
    #[description_localized("ja", "チャンネル種別")]
    channel_type: String,

    #[name_localized("ja", "チャンネル")]
    #[description = "Channel"]
    #[description_localized("ja", "登録するチャンネル")]
    channel: Channel,
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

    // チャンネルIDを取得
    let channel_id = channel.id().get();

    // ギルド名を取得
    let guild_name = ctx
        .guild()
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Unknown Guild".to_string());

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = ChannelManagementFacade::new(Arc::new(app_state.clone()));
    let result = facade
        .register_channel(
            guild_id.get() as i64,
            guild_name,
            channel_type_id,
            channel_id as i64,
        )
        .await?;

    // 結果メッセージを作成
    let mut message = format!(
        "✅ チャンネルを登録しました。\n\n**種別:** {}\n**チャンネル:** <#{}>\n\n**現在の設定状況:**\n",
        result.channel_type_name, result.channel_id
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
