/// グローバルスプレッドシート読み込みコマンド
///
/// Bot管理者専用サーバーでのみ実行可能
/// スプレッドシートからグローバルデータをPostgreSQLに読み込みます
use crate::errors::PresentationError;
use crate::events::permission::check_bot_admin_server;
use crate::facades::spreadsheet::SpreadsheetImportFacade;
use crate::services::message::MessageTextId;
use crate::events::helpers::get_message_from_context;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::{error, info};

#[poise::command(
    slash_command,
    check = "check_bot_admin_server",
    ephemeral = true,
    name_localized("ja", "グローバルスプレッドシート読み込み"),
    description_localized(
        "ja",
        "グローバルスプレッドシートからデータ読み込み（管理者専用サーバー）"
    )
)]
pub async fn gspread_global_load(ctx: PoiseContext<'_>) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer_ephemeral().await?;

    info!(
        user_id = %ctx.author().id,
        "グローバルスプレッドシート読み込みを開始"
    );

    // 環境変数からスプレッドシートIDを取得
    let spreadsheet_id = match env::var("GLOBAL_SPREADSHEET_ID") {
        Ok(id) => id,
        Err(_) => {
            // 新しいメッセージサービスを使用
            let mut params = HashMap::new();
            params.insert("var_name".to_string(), "GLOBAL_SPREADSHEET_ID".to_string());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ErrorsEnvVarNotSet,
                params,
            )
            .await
            .unwrap_or_else(|_| {
                "❌ エラー: 環境変数 GLOBAL_SPREADSHEET_ID が設定されていません".to_string()
            });

            ctx.say(&message).await?;
            error!("環境変数 GLOBAL_SPREADSHEET_ID が設定されていません");
            return Ok(());
        }
    };

    ctx.say("🔄 グローバルスプレッドシートからデータを読み込んでいます...")
        .await?;

    // Facadeを作成（Global ロールを使用 - master スキーマへの書き込み権限が必要）
    let app_state = Arc::new(ctx.data().app_state.clone());
    let facade =
        match SpreadsheetImportFacade::new(app_state.global_db().clone(), app_state.clone()) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = PresentationError::from(e).to_string();
                ctx.say(format!("❌ {error_msg}")).await?;
                return Ok(());
            }
        };

    // インポート実行
    match facade.import_global_spreadsheet(&spreadsheet_id).await {
        Ok(result) => {
            // Display実装を使用してメッセージを生成
            let message = format!("✅ グローバルスプレッドシート読み込み完了\n\n{result}");

            ctx.say(message).await?;

            info!(
                success = result.success_count,
                failure = result.failure_count,
                total_rows = result.total_rows,
                "グローバルスプレッドシート読み込み完了"
            );

            Ok(())
        }
        Err(e) => {
            error!(
                user_id = %ctx.author().id,
                error = %e,
                error_debug = ?e,
                "グローバルスプレッドシート読み込み中にFacadeエラーが発生しました"
            );

            let error_msg = PresentationError::from(e).to_string();
            ctx.say(format!(
                "❌ グローバルスプレッドシート読み込み失敗\n\n{error_msg}"
            ))
            .await?;

            Ok(())
        }
    }
}
