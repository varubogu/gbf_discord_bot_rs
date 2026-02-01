/// ギルドスプレッドシート登録コマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// ギルド用の読み込み・書き込みスプレッドシートをデータベースに登録します
use crate::errors::PresentationError;
use crate::events::permission::check_bot_control_role;
use crate::facades::spreadsheet::GuildSpreadsheetRegistrationFacade;
use crate::services::message::MessageTextId;
use crate::services::message::helpers::get_message_from_context;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use tracing::{error, info};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    name_localized("ja", "スプレッドシート登録"),
    description_localized("ja", "ギルド用のGoogleスプレッドシートを登録")
)]
pub async fn gspread_register(
    ctx: PoiseContext<'_>,

    #[max_length = 512]
    #[name_localized("ja", "読み込み用スプレッドシート")]
    #[description = "Read-only spreadsheet URL (or ID)"]
    #[description_localized("ja", "読み込み用スプレッドシートURL（またはID）")]
    load_spreadsheet_url: String,

    #[max_length = 512]
    #[name_localized("ja", "書き込み用スプレッドシート")]
    #[description = "Write-only spreadsheet URL (or ID)"]
    #[description_localized("ja", "書き込み用スプレッドシートURL（またはID）")]
    push_spreadsheet_url: String,
) -> Result<()> {
    // 即座にdeferして処理時間を確保
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ErrorsGuildOnly,
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
        "ギルドスプレッドシート登録を開始"
    );

    let loading_message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::SpreadsheetRegistering,
        HashMap::new(),
    )
    .await
    .unwrap_or_else(|_| "🔄 ギルドスプレッドシートを登録しています...".to_string());

    ctx.say(&loading_message).await?;

    // Facadeを作成
    let app_state = &ctx.data().app_state;
    let facade = match GuildSpreadsheetRegistrationFacade::new(app_state.guild_db().clone()) {
        Ok(f) => f,
        Err(e) => {
            let error_msg = PresentationError::from(e).to_string();
            let mut params = HashMap::new();
            params.insert("error_msg".to_string(), error_msg.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::SpreadsheetRegisterFailed,
                params,
            )
            .await
            .unwrap_or_else(|_| format!("❌ {error_msg}"));

            ctx.say(&message).await?;
            return Ok(());
        }
    };

    // 登録実行
    match facade
        .register_guild_spreadsheets(guild_id, &load_spreadsheet_url, &push_spreadsheet_url)
        .await
    {
        Ok(result) => {
            let mut params = HashMap::new();
            params.insert("load_url".to_string(), result.load_spreadsheet_url.clone());
            params.insert("push_url".to_string(), result.push_spreadsheet_url.clone());

            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::SpreadsheetRegisterSuccess,
                params,
            )
            .await
            .unwrap_or_else(|_| {
                format!(
                    "✅ ギルドスプレッドシートを登録しました\n\n\
                     📊 登録内容:\n\
                     - 読み込み用: <{}>\n\
                     - 書き込み用: <{}>\n\n\
                     これで `/gspread_load` と `/gspread_push` コマンドが使用可能になりました。",
                    result.load_spreadsheet_url, result.push_spreadsheet_url
                )
            });

            ctx.say(&message).await?;

            info!(
                guild_id = %guild_id,
                load_url = %result.load_spreadsheet_url,
                push_url = %result.push_spreadsheet_url,
                "ギルドスプレッドシート登録完了"
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
                MessageTextId::SpreadsheetRegisterFailed,
                params,
            )
            .await
            .unwrap_or_else(|_| format!("❌ ギルドスプレッドシート登録失敗\n\n{error_msg}"));

            ctx.say(&message).await?;

            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシート登録失敗"
            );
            Ok(())
        }
    }
}
