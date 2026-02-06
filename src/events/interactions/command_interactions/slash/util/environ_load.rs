use crate::facades::environment;
use crate::types::{PoiseContext, Result};

#[poise::command(
    slash_command,
    ephemeral = true,
    name_localized("ja", "設定値リロード"),
    description_localized("ja", "Botの設定値をサーバーから読み込みます")
)]
pub async fn environ_load(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    match environment::load(guild_id.get()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
