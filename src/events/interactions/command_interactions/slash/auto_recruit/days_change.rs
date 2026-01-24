//! 自動募集日数変更コマンド

use crate::facades::auto_recruitment;
use crate::services::message::MessageTextId;
use crate::services::permission::check_bot_control_role;
use crate::types::{AppError, PoiseContext, Result};
use rust_i18n::t;
use tracing::error;

/// 自動募集の募集日数を変更
///
/// 自動募集の募集日数を変更します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "auto_recruit_days_change",
    name_localized("ja", "自動募集日数変更"),
    description_localized("ja", "自動募集の募集日数を変更します（gbf_bot_controlロール必須）")
)]
pub async fn auto_recruit_days_change(
    ctx: PoiseContext<'_>,

    #[name_localized("ja", "募集日数")]
    #[description = "Days range (2-7)"]
    #[description_localized("ja", "新しい募集日数（2〜7日）")]
    #[min = 2]
    #[max = 7]
    days: i32,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    let app_state = &ctx.data().app_state;
    let serenity_ctx = ctx.serenity_context();

    match auto_recruitment::change_days(serenity_ctx, app_state, guild_id.get(), days).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("✅ 自動募集の募集日数を{}日に変更しました。", days))
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id = guild_id.get(), "自動募集日数の変更に失敗しました");

            // エラーメッセージを多言語対応で取得
            let error_message = match &e {
                AppError::ChannelCreationFailed => {
                    let locale = ctx.locale().unwrap_or("ja");
                    t!(
                        MessageTextId::AutoRecruitmentChannelCreateFailed.as_str(),
                        locale = locale
                    )
                    .to_string()
                }
                _ => format!("エラー: {}", e),
            };

            ctx.send(
                poise::CreateReply::default()
                    .content(error_message)
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
