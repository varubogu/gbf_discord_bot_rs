use crate::facades::timezone::TimezoneFacade;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use std::sync::Arc;

use super::autocomplete::timezone_auto_complete;

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "タイムゾーン設定"),
    description_localized("ja", "サーバーのタイムゾーンを設定します")
)]
pub async fn timezone_set(
    ctx: PoiseContext<'_>,

    #[autocomplete = "timezone_auto_complete"]
    #[name_localized("ja", "タイムゾーン")]
    #[description = "timezone"]
    #[description_localized("ja", "タイムゾーン（選択式）")]
    timezone: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string()))?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let facade = TimezoneFacade::new(Arc::new(app_state.clone()));
    let result = facade.set_timezone(guild_id.get() as i64, &timezone).await?;

    // 成功メッセージ
    ctx.say(format!("タイムゾーンを {} に設定しました。", result.timezone.name()))
        .await?;

    Ok(())
}
