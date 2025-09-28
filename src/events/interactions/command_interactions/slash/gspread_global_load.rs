use crate::types::{PoiseContext, Result};
use crate::facades::spreadsheet::execute_global_load;

#[poise::command(
    slash_command,
    name_localized("ja", "グローバル読み込み"),
    description_localized("ja", "スプレッドシートからデータ読み込み（管理者専用サーバー）")
)]
pub async fn gspread_global_load(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;
    execute_global_load(&ctx).await
}
