use crate::types::{PoiseContext, Result};
use crate::facades::spreadsheet::execute_global_push;

#[poise::command(
    slash_command,
    name_localized("ja", "グローバル書き込み"),
    description_localized("ja", "データベースからスプレッドシートへ書き込み（管理者専用サーバー）")
)]
pub async fn gspread_global_push(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer().await?;
    execute_global_push(&ctx).await
}
