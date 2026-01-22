//! 自動募集カテゴリ解除コマンド

use crate::facades::auto_recruitment;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use tracing::error;

/// 自動募集カテゴリを解除
///
/// このギルドの自動募集設定を解除し、日時チャンネルを削除します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "auto_recruit_category_unregister",
    name_localized("ja", "自動募集カテゴリ解除"),
    description_localized(
        "ja",
        "このギルドの自動募集設定を解除します（gbf_bot_controlロール必須）"
    )
)]
pub async fn auto_recruit_category_unregister(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    let app_state = &ctx.data().app_state;
    let serenity_ctx = ctx.serenity_context();

    match auto_recruitment::unregister_category(serenity_ctx, app_state, guild_id.get()).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(
                        "✅ 自動募集カテゴリを解除しました。関連するチャンネルも削除されました。",
                    )
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id = guild_id.get(), "自動募集カテゴリの解除に失敗しました");
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("エラー: {}", e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
