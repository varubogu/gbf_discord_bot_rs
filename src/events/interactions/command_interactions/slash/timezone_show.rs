use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::permission::check_bot_control_role;
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use std::sync::Arc;

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "タイムゾーン確認"),
    description_localized("ja", "サーバーの現在のタイムゾーン設定を確認します")
)]
pub async fn timezone_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string()))?;

    // タイムゾーンを取得
    let app_state = ctx.data();
    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    let timezone_service = TimezoneService::new(timezone_repo.clone());
    let timezone = timezone_service
        .get_guild_timezone(app_state.app_state.guild_db(), guild_id.get() as i64)
        .await?;

    // DB設定があるか確認
    let is_default = timezone_repo
        .find_by_guild_id(app_state.app_state.guild_db(), guild_id.get() as i64)
        .await?
        .is_none();

    // 結果メッセージ
    let message = if is_default {
        format!(
            "現在のタイムゾーン: {}\nデフォルト設定: はい",
            timezone.name()
        )
    } else {
        format!(
            "現在のタイムゾーン: {}\nデフォルト設定: いいえ",
            timezone.name()
        )
    };

    ctx.say(message).await?;

    Ok(())
}
