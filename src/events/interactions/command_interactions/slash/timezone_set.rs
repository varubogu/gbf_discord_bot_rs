use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::permission::check_bot_control_role;
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
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

    // タイムゾーン名のバリデーション
    let tz = TimezoneService::validate_timezone(&timezone)?;

    // タイムゾーンを設定
    let app_state = ctx.data();
    let db = app_state.app_state.guild_db();
    let txn = db.begin().await?;

    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    timezone_repo
        .upsert_with_txn(&txn, guild_id.get() as i64, tz.name())
        .await?;

    txn.commit().await?;

    // 成功メッセージ
    ctx.say(format!("タイムゾーンを {} に設定しました。", tz.name()))
        .await?;

    Ok(())
}
