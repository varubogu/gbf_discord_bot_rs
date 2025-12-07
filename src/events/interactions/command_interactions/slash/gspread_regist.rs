/// ギルドスプレッドシート登録コマンド
///
/// gbf_bot_controlロール保持者が実行可能
/// ギルド用の読み込み・書き込みスプレッドシートをデータベースに登録します
use crate::errors::PresentationError;
use crate::facades::spreadsheet::GuildSpreadsheetRegistrationFacade;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use tracing::{error, info};

#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    name_localized("ja", "スプレッドシート登録"),
    description_localized("ja", "ギルド用のGoogleスプレッドシートを登録")
)]
pub async fn gspread_regist(
    ctx: PoiseContext<'_>,
    #[description = "読み込み用スプレッドシートURL（またはID）"]
    #[name_localized("ja", "読み込み用スプレッドシート")]
    #[max_length = 512]
    load_spreadsheet_url: String,
    #[description = "書き込み用スプレッドシートURL（またはID）"]
    #[name_localized("ja", "書き込み用スプレッドシート")]
    #[max_length = 512]
    push_spreadsheet_url: String,
) -> Result<()> {
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
        "ギルドスプレッドシート登録を開始"
    );

    ctx.send(
        poise::CreateReply::default()
            .content("🔄 ギルドスプレッドシートを登録しています...")
            .ephemeral(true),
    )
        .await?;

    // Facadeを作成
    let app_state = &ctx.data().app_state;
    let facade = match GuildSpreadsheetRegistrationFacade::new(app_state.guild_db().clone()) {
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

    // 登録実行
    match facade
        .register_guild_spreadsheets(guild_id, &load_spreadsheet_url, &push_spreadsheet_url)
        .await
    {
        Ok(result) => {
            let message = format!(
                "✅ ギルドスプレッドシートを登録しました\n\n\
                 📊 登録内容:\n\
                 - 読み込み用: <{}>\n\
                 - 書き込み用: <{}>\n\n\
                 これで `/gspread_load` と `/gspread_push` コマンドが使用可能になりました。",
                result.load_spreadsheet_url, result.push_spreadsheet_url
            );

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;

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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ ギルドスプレッドシート登録失敗\n\n{}", error_msg))
                    .ephemeral(true),
            )
            .await?;
            error!(
                guild_id = %guild_id,
                "ギルドスプレッドシート登録失敗"
            );
            Ok(())
        }
    }
}
