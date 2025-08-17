use crate::facades::environment;
use crate::types::{PoiseContext, PoiseError};

#[poise::command(
    slash_command,
    name_localized("ja", "設定値リロード"),
    description_localized("ja", "Botの設定値をサーバーから読み込みます"),
    ephemeral
)]
pub async fn environ_load(ctx: PoiseContext<'_>) -> Result<(), PoiseError> {
    ctx.defer().await?;
    match environment::load(&ctx).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(e),
    }
}
