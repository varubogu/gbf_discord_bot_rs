use crate::types::{PoiseContext, Result};
use crate::facades::spreadsheet::execute_push;

#[poise::command(
    slash_command,
    name_localized("ja", "書き込み"),
    description_localized("ja", "データベースからサーバー固有スプレッドシートへ書き込み")
)]
pub async fn gspread_push(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;
    execute_push(&ctx).await
}
