/// ギルドスプレッドシート読み込みコマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// スプレッドシートからギルドデータをPostgreSQLに読み込みます
use crate::errors::PresentationError;
use crate::facades::spreadsheet::SpreadsheetImportFacade;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::{GuildSpreadsheetConfigRepository, GuildSpreadsheetConfigRepositoryTrait};
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
use std::sync::Arc;
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
        "ギルドスプレッドシート読み込みを開始"
    );

    // データベースからギルドの読み込み用スプレッドシートIDを取得
    let app_state = &ctx.data().app_state;
    let db = app_state.guild_db();

    // RLSポリシーのためにトランザクションを開始してセッション変数を設定
    let txn = db.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let repository = GuildSpreadsheetConfigRepository::new();

    let spreadsheet_id = match GuildSpreadsheetConfigRepositoryTrait::find_import_spreadsheet_id(&repository, &txn, guild_id).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            txn.rollback().await?;
            ctx.send(
                poise::CreateReply::default()
                    .content("❌ エラー: このギルドにスプレッドシートが登録されていません\n\
                        `/gspread_regist` コマンドでスプレッドシートを登録してください")
                    .ephemeral(true),
            )
            .await?;
            error!(
                guild_id = %guild_id,
                "ギルド読み込み用スプレッドシートIDが設定されていません"
            );
            return Ok(());
        }
        Err(e) => {
            txn.rollback().await?;
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ エラー: スプレッドシート設定の取得に失敗しました\n{}", e))
                    .ephemeral(true),
            )
            .await?;
            error!(
                guild_id = %guild_id,
                error = %e,
                "スプレッドシート設定の取得に失敗"
            );
            return Ok(());
        }
    };

    // トランザクションをコミット（スプレッドシートID取得が成功）
    txn.commit().await?;

    ctx.say("🔄 ギルドスプレッドシートからデータを読み込んでいます...")
        .await?;

    // Facadeを作成
    let app_state_arc = Arc::new(app_state.clone());
    let facade = match SpreadsheetImportFacade::new(app_state.guild_db().clone(), app_state_arc.clone())
    {
        Ok(f) => f,
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ {}", error_msg))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // インポート実行（ギルド版のメソッドを呼び出す）
    match facade.import_guild_spreadsheet(&spreadsheet_id, guild_id as u64).await {
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

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;

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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ ギルドスプレッドシート読み込み失敗\n\n{}", error_msg))
                    .ephemeral(true),
            )
            .await?;

            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシート読み込み失敗"
            );

            Ok(())
        }
    }
}
