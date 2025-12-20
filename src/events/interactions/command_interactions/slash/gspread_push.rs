/// ギルドスプレッドシート書き込みコマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// PostgreSQLからギルドデータをスプレッドシートに書き込みます
use crate::errors::PresentationError;
use crate::facades::spreadsheet::SpreadsheetExportFacade;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use tracing::{error, info};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スプレッドシート書き込み"),
    description_localized("ja", "ギルドデータをスプレッドシートに書き込み")
)]
pub async fn gspread_push(ctx: PoiseContext<'_>) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer().await?;

    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("❌ このコマンドはギルド内でのみ実行可能です")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    info!(
        user_id = %ctx.author().id,
        guild_id = %guild_id,
        "ギルドデータのスプレッドシート書き込みを開始"
    );

    let app_state = &ctx.data().app_state;
    ctx.send(
        poise::CreateReply::default()
            .content("🔄 ギルドデータをスプレッドシートに書き込んでいます...")
            .ephemeral(true),
    )
    .await?;

    // Facadeを作成
    let facade = match SpreadsheetExportFacade::new(app_state.guild_db().clone()) {
        Ok(f) => f,
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ {error_msg}"))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // エクスポート実行（Facade内でTx管理と設定取得を行う）
    match facade.export_for_guild_by_config(guild_id).await {
        Ok(result) => {
            let message = if result.failure_count == 0 && result.errors.is_empty() {
                format!(
                    "✅ ギルドデータ書き込み完了\n\n\
                     📊 書き込み結果:\n\
                     - 成功: {}テーブル\n\
                     - 総行数: {}行",
                    result.success_count, result.total_rows
                )
            } else {
                format!(
                    "⚠️ ギルドデータ書き込み完了（一部エラー）\n\n\
                     📊 書き込み結果:\n\
                     - 成功: {}テーブル\n\
                     - 失敗: {}テーブル\n\
                     - 総行数: {}行\n\n\
                     {}",
                    result.success_count,
                    result.failure_count,
                    result.total_rows,
                    if result.errors.len() <= 5 {
                        format!("❌ エラー:\n{}", result.errors.join("\n"))
                    } else {
                        format!(
                            "❌ エラー（最初の5件）:\n{}\n... 他{}件",
                            result.errors[..5].join("\n"),
                            result.errors.len() - 5
                        )
                    }
                )
            };

            ctx.say(message).await?;

            info!(
                guild_id = %guild_id,
                success = result.success_count,
                failure = result.failure_count,
                total_rows = result.total_rows,
                "ギルドデータ書き込み完了"
            );

            Ok(())
        }
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ ギルドデータ書き込み失敗\n\n{error_msg}"))
                    .ephemeral(true),
            )
            .await?;

            error!(
                guild_id = %guild_id,
                "ギルドデータ書き込み失敗"
            );

            Ok(())
        }
    }
}
