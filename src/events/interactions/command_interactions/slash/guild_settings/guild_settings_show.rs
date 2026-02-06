use crate::events::helpers::get_message_from_context;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use std::sync::Arc;

#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    name_localized("ja", "サーバー設定確認"),
    description_localized("ja", "サーバーの現在の設定（タイムゾーン、ロケールを確認します")
)]
pub async fn guild_settings_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    // ギルド設定を取得（Facade経由）
    let app_state = &ctx.data().app_state;
    let facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let settings = facade.get_guild_settings(guild_id.get() as i64).await?;

    // 結果メッセージ
    let message = match settings {
        Some(s) => {
            let mut params = HashMap::new();
            params.insert("timezone".to_string(), s.timezone.clone());
            params.insert("locale".to_string(), s.locale.clone());

            get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::GuildSettingsShowSuccess,
                params,
            )
            .await
            .unwrap_or_else(|_| {
                format!(
                    "現在のサーバー設定:\nタイムゾーン: {}\n言語: {}",
                    s.timezone, s.locale
                )
            })
        }
        None => get_message_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::GuildSettingsNotSet,
            HashMap::new(),
        )
        .await
        .unwrap_or_else(|_| "サーバー設定がされていません。".to_string()),
    };

    ctx.say(&message).await?;

    Ok(())
}
