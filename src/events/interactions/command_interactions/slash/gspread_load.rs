use crate::types::{PoiseContext, Result};
use crate::facades::spreadsheet::execute_load;

#[poise::command(
    slash_command,
    name_localized("ja", "読み込み"),
    description_localized("ja", "サーバー固有スプレッドシートからデータ読み込み")
)]
pub async fn gspread_load(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;
    execute_load(&ctx).await
}
