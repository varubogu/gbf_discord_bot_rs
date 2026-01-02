use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::message::MessageTextId;
use crate::services::message::helpers::get_message_from_context;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::super::autocomplete::{locale_auto_complete, timezone_auto_complete};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "サーバー設定"),
    description_localized("ja", "サーバーのタイムゾーンと言語を設定します")
)]
pub async fn guild_settings_set(
    ctx: PoiseContext<'_>,

    #[autocomplete = "timezone_auto_complete"]
    #[name_localized("ja", "タイムゾーン")]
    #[description = "timezone"]
    #[description_localized("ja", "タイムゾーン（選択式）")]
    timezone: String,

    #[autocomplete = "locale_auto_complete"]
    #[name_localized("ja", "言語")]
    #[description = "locale (ja or en)"]
    #[description_localized("ja", "言語（ja: 日本語、en: 英語）")]
    locale: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let result = facade
        .set_timezone(guild_id.get() as i64, &timezone, &locale)
        .await?;

    // 成功メッセージ
    let mut params = HashMap::new();
    params.insert("timezone".to_string(), result.timezone.name().to_string());
    params.insert("locale".to_string(), locale.to_string());

    let message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::GuildSettingsSetSuccess,
        params,
    )
    .await
    .unwrap_or_else(|_| {
        format!(
            "サーバー設定を更新しました。\nタイムゾーン: {}\n言語: {}",
            result.timezone.name(),
            locale
        )
    });

    ctx.say(&message).await?;

    Ok(())
}
