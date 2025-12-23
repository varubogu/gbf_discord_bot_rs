/// ギルドスプレッドシート読み込みコマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// スプレッドシートからギルドデータをPostgreSQLに読み込みます
use crate::errors::PresentationError;
use crate::facades::spreadsheet::SpreadsheetImportFacade;
use crate::services::message::helpers::get_message_from_context;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スプレッドシート読み込み"),
    description_localized("ja", "ギルドスプレッドシートからデータ読み込み")
)]
pub async fn gspread_load(ctx: PoiseContext<'_>) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                "errors.guild_only",
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "❌ このコマンドはギルド内でのみ実行可能です".to_string());

            ctx.say(&message).await?;
            return Ok(());
        }
    };

    info!(
        user_id = %ctx.author().id,
        guild_id = %guild_id,
        "ギルドスプレッドシート読み込みを開始"
    );

    let app_state = &ctx.data().app_state;

    // Facadeを作成
    let app_state_arc = Arc::new(app_state.clone());
    let facade =
        match SpreadsheetImportFacade::new(app_state.guild_db().clone(), app_state_arc.clone()) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = PresentationError::from(e).to_string();
                ctx.say(format!("❌ {error_msg}")).await?;
                return Ok(());
            }
        };

    // Facadeを使ってスプレッドシートIDを取得
    let spreadsheet_id = match facade.get_guild_spreadsheet_id(guild_id).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                "errors.spreadsheet_not_registered",
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                "❌ エラー: このギルドにスプレッドシートが登録されていません\n\
                `/gspread_regist` コマンドでスプレッドシートを登録してください"
                    .to_string()
            });

            ctx.say(&message).await?;
            error!(
                guild_id = %guild_id,
                "ギルド読み込み用スプレッドシートIDが設定されていません"
            );
            return Ok(());
        }
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), error_msg.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                "errors.spreadsheet_config_fetch_failed",
                params,
            )
            .await
            .unwrap_or_else(|_| {
                format!("❌ エラー: スプレッドシート設定の取得に失敗しました\n{error_msg}")
            });

            ctx.say(&message).await?;
            error!(
                guild_id = %guild_id,
                "スプレッドシート設定の取得に失敗"
            );
            return Ok(());
        }
    };

    let loading_message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        "spreadsheet.loading",
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| "🔄 ギルドスプレッドシートからデータを読み込んでいます...".to_string());

    ctx.say(&loading_message).await?;

    // インポート実行（ギルド版のメソッドを呼び出す）
    match facade
        .import_guild_spreadsheet(&spreadsheet_id, guild_id as u64)
        .await
    {
        Ok(result) => {
            let message = if result.failure_count == 0 && result.errors.is_empty() {
                let mut params = HashMap::new();
                params.insert("success_count".to_string(), result.success_count.to_string());
                params.insert("total_rows".to_string(), result.total_rows.to_string());

                get_message_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    "spreadsheet.load_success",
                    params,
                )
                .await
                .unwrap_or_else(|_| {
                    format!(
                        "✅ ギルドスプレッドシート読み込み完了\n\n\
                         📊 読み込み結果:\n\
                         - 成功: {}テーブル\n\
                         - 総行数: {}行",
                        result.success_count, result.total_rows
                    )
                })
            } else {
                let error_details = if result.errors.len() <= 5 {
                    format!("❌ エラー:\n{}", result.errors.join("\n"))
                } else {
                    format!(
                        "❌ エラー（最初の5件）:\n{}\n... 他{}件",
                        result.errors[..5].join("\n"),
                        result.errors.len() - 5
                    )
                };

                let mut params = HashMap::new();
                params.insert("success_count".to_string(), result.success_count.to_string());
                params.insert("failure_count".to_string(), result.failure_count.to_string());
                params.insert("total_rows".to_string(), result.total_rows.to_string());
                params.insert("error_details".to_string(), error_details.clone());

                get_message_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    "spreadsheet.load_partial_success",
                    params,
                )
                .await
                .unwrap_or_else(|_| {
                    format!(
                        "⚠️ ギルドスプレッドシート読み込み完了（一部エラー）\n\n\
                         📊 読み込み結果:\n\
                         - 成功: {}テーブル\n\
                         - 失敗: {}テーブル\n\
                         - 総行数: {}行\n\n\
                         {}",
                        result.success_count, result.failure_count, result.total_rows, error_details
                    )
                })
            };

            ctx.say(&message).await?;

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
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), error_msg.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                "spreadsheet.load_failed",
                params,
            )
            .await
            .unwrap_or_else(|_| format!("❌ ギルドスプレッドシート読み込み失敗\n\n{error_msg}"));

            ctx.say(&message).await?;

            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシート読み込み失敗"
            );

            Ok(())
        }
    }
}
