use crate::facades::environment;
use crate::types::{PoiseContext, Result};

#[poise::command(
    slash_command,
    ephemeral = true,
    name_localized("ja", "設定値リロード"),
    description_localized("ja", "Botの設定値をサーバーから読み込みます"),
)]
pub async fn environ_load(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;
    match environment::load(&ctx).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
