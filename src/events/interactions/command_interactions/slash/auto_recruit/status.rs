//! 自動募集参加状況確認コマンド

use crate::facades::auto_recruitment;
use crate::types::{PoiseContext, Result};
use tracing::error;

/// 自動募集の参加状況を確認
///
/// 自分の自動募集への参加状況を確認します。
#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    rename = "auto_recruit_status",
    name_localized("ja", "自動募集参加状況"),
    description_localized("ja", "自分の自動募集への参加状況を確認します")
)]
pub async fn auto_recruit_status(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    let user_id = ctx.author().id.get();
    let app_state = &ctx.data().app_state;

    match auto_recruitment::get_participation_status(app_state, guild_id.get(), user_id).await {
        Ok(status) => {
            let mut message = String::from("**自動募集参加状況**\n\n");

            // クエスト選択状況
            if status.quest_ids.is_empty() {
                message.push_str("**選択中のクエスト:** なし\n\n");
            } else {
                message.push_str(&format!(
                    "**選択中のクエスト:** {}個\n",
                    status.quest_ids.len()
                ));
                // TODO: クエスト名を取得して表示
                message.push_str(&format!(
                    "クエストID: {}\n\n",
                    status
                        .quest_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            // 時間選択状況
            if status.time_slots.is_empty() {
                message.push_str("**参加可能時間:** なし");
            } else {
                message.push_str("**参加可能時間:**\n");
                for slot in &status.time_slots {
                    let hours_str = slot
                        .hours
                        .iter()
                        .map(|h| format!("{}時", h))
                        .collect::<Vec<_>>()
                        .join(", ");
                    message.push_str(&format!(
                        "• {}月{}日: {}\n",
                        slot.month, slot.day, hours_str
                    ));
                }
            }

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id = guild_id.get(), user_id, "参加状況の取得に失敗しました");
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
