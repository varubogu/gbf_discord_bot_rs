/// ギルドスプレッドシート書き込みコマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// PostgreSQLからギルドデータをスプレッドシートに書き込みます
use crate::errors::PresentationError;
use crate::facades::spreadsheet::SpreadsheetExportFacade;
use crate::services::message::MessageId;
use crate::services::message::helpers::get_message_from_context;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
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
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::ErrorsGuildOnly,
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
        "ギルドデータのスプレッドシート書き込みを開始"
    );

    let app_state = &ctx.data().app_state;

    let loading_message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageId::SpreadsheetPushing,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| "🔄 ギルドデータをスプレッドシートに書き込んでいます...".to_string());

    ctx.say(&loading_message).await?;

    // Facadeを作成
    let facade = match SpreadsheetExportFacade::new(app_state.guild_db().clone()) {
        Ok(f) => f,
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), error_msg.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::SpreadsheetPushFailed,
                params,
            )
            .await
            .unwrap_or_else(|_| format!("❌ {error_msg}"));

            ctx.say(&message).await?;
            return Ok(());
        }
    };

    // エクスポート実行（Facade内でTx管理と設定取得を行う）
    match facade.export_for_guild_by_config(guild_id).await {
        Ok(result) => {
            let message = if result.failure_count == 0 && result.errors.is_empty() {
                let mut params = HashMap::new();
                params.insert(
                    "success_count".to_string(),
                    result.success_count.to_string(),
                );
                params.insert("total_rows".to_string(), result.total_rows.to_string());

                get_message_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageId::SpreadsheetPushSuccess,
                    params,
                )
                .await
                .unwrap_or_else(|_| {
                    format!(
                        "✅ ギルドデータ書き込み完了\n\n\
                         📊 書き込み結果:\n\
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
                params.insert(
                    "success_count".to_string(),
                    result.success_count.to_string(),
                );
                params.insert(
                    "failure_count".to_string(),
                    result.failure_count.to_string(),
                );
                params.insert("total_rows".to_string(), result.total_rows.to_string());
                params.insert("error_details".to_string(), error_details.clone());

                get_message_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageId::SpreadsheetPushPartialSuccess,
                    params,
                )
                .await
                .unwrap_or_else(|_| {
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
                        error_details
                    )
                })
            };

            ctx.say(&message).await?;

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
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), error_msg.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::SpreadsheetPushFailed,
                params,
            )
            .await
            .unwrap_or_else(|_| format!("❌ ギルドデータ書き込み失敗\n\n{error_msg}"));

            ctx.say(&message).await?;

            error!(
                guild_id = %guild_id,
                "ギルドデータ書き込み失敗"
            );

            Ok(())
        }
    }
}
