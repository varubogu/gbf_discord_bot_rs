/// ギルドスプレッドシート読み込みコマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// スプレッドシートからギルドデータをPostgreSQLに読み込みます

use crate::errors::PresentationError;
use crate::facades::spreadsheet::SpreadsheetImportFacade;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use sea_orm::prelude::*;
use tracing::{error, info};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    name_localized("ja", "読み込み"),
    description_localized("ja", "ギルドスプレッドシートからデータ読み込み")
)]
pub async fn gspread_load(ctx: PoiseContext<'_>) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer().await?;

    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("❌ このコマンドはギルド内でのみ実行可能です")
                .await?;
            return Ok(());
        }
    };

    info!(
        user_id = %ctx.author().id,
        guild_id = %guild_id,
        "ギルドスプレッドシート読み込みを開始"
    );

    // TODO: データベースからギルドのスプレッドシートIDを取得
    // 現時点では仮実装（常にエラーを返す）
    // 実装例: guild_environmentsテーブルから key="spreadsheet_id" で取得
    let spreadsheet_id: Option<String> = None; // 仮実装

    let spreadsheet_id = match spreadsheet_id {
        Some(id) => id,
        None => {
            ctx.say(
                "❌ エラー: このギルドにスプレッドシートが設定されていません\n\
                 `/environ_load` コマンドでギルド設定を確認してください",
            )
            .await?;
            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシートIDが設定されていません"
            );
            return Ok(());
        }
    };

    ctx.say("🔄 ギルドスプレッドシートからデータを読み込んでいます...")
        .await?;

    // Facadeを作成
    let app_state = &ctx.data().app_state;
    let facade = match SpreadsheetImportFacade::new(app_state.db().clone()) {
        Ok(f) => f,
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            ctx.say(format!("❌ {}", error_msg)).await?;
            return Ok(());
        }
    };

    // インポート実行
    match facade
        .import_guild_spreadsheet(&spreadsheet_id, guild_id)
        .await
    {
        Ok(result) => {
            let message = if result.failure_count == 0 && result.errors.is_empty() {
                format!(
                    "✅ ギルドスプレッドシート読み込み完了\n\n\
                     📊 読み込み結果:\n\
                     - 成功: {}テーブル\n\
                     - 総行数: {}行",
                    result.success_count, result.total_rows
                )
            } else {
                format!(
                    "⚠️ ギルドスプレッドシート読み込み完了（一部エラー）\n\n\
                     📊 読み込み結果:\n\
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
                "ギルドスプレッドシート読み込み完了"
            );

            Ok(())
        }
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            ctx.say(format!("❌ ギルドスプレッドシート読み込み失敗\n\n{}", error_msg))
                .await?;

            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシート読み込み失敗"
            );

            Ok(())
        }
    }
}
