//! 自動募集参加状況確認コマンド

use crate::events::helpers::get_message_from_context;
use crate::facades::auto_recruitment;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use tracing::error;

async fn get_message_or_key(
    ctx: &PoiseContext<'_>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> String {
    get_message_from_context(
        ctx,
        ctx.data().app_state.message_service(),
        message_id,
        params,
    )
    .await
    .unwrap_or_else(|_| message_id.as_str().to_string())
}

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
            message: MessageTextId::ErrorsGuildOnly.as_str().to_string(),
        })?;

    let user_id = ctx.author().id.get();
    let app_state = &ctx.data().app_state;

    match auto_recruitment::get_participation_status(app_state, guild_id.get(), user_id).await {
        Ok(status) => {
            let mut message = String::new();
            let header = get_message_or_key(
                &ctx,
                MessageTextId::AutoRecruitmentStatusHeader,
                HashMap::new(),
            )
            .await;
            message.push_str(&header);
            message.push_str("\n\n");

            // クエスト選択状況
            if status.quest_ids.is_empty() {
                let quest_empty = get_message_or_key(
                    &ctx,
                    MessageTextId::AutoRecruitmentStatusQuestEmpty,
                    HashMap::new(),
                )
                .await;
                message.push_str(&quest_empty);
                message.push_str("\n\n");
            } else {
                let mut params = HashMap::new();
                params.insert("count".to_string(), status.quest_ids.len().to_string());
                let quest_count = get_message_or_key(
                    &ctx,
                    MessageTextId::AutoRecruitmentStatusQuestCount,
                    params,
                )
                .await;
                message.push_str(&quest_count);
                message.push('\n');

                // TODO: クエスト名を取得して表示
                let mut params = HashMap::new();
                params.insert(
                    "quest_ids".to_string(),
                    status
                        .quest_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                let quest_ids =
                    get_message_or_key(&ctx, MessageTextId::AutoRecruitmentStatusQuestIds, params)
                        .await;
                message.push_str(&quest_ids);
                message.push_str("\n\n");
            }

            // 時間選択状況
            if status.time_slots.is_empty() {
                let time_empty = get_message_or_key(
                    &ctx,
                    MessageTextId::AutoRecruitmentStatusTimeEmpty,
                    HashMap::new(),
                )
                .await;
                message.push_str(&time_empty);
            } else {
                let time_header = get_message_or_key(
                    &ctx,
                    MessageTextId::AutoRecruitmentStatusTimeHeader,
                    HashMap::new(),
                )
                .await;
                message.push_str(&time_header);
                message.push('\n');
                for slot in &status.time_slots {
                    let hours_str = slot
                        .hours
                        .iter()
                        .map(|h| format!("{h}時"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let mut params = HashMap::new();
                    params.insert("month".to_string(), slot.month.to_string());
                    params.insert("day".to_string(), slot.day.to_string());
                    params.insert("hours_str".to_string(), hours_str);
                    let time_slot = get_message_or_key(
                        &ctx,
                        MessageTextId::AutoRecruitmentStatusTimeSlot,
                        params,
                    )
                    .await;
                    message.push_str(&time_slot);
                    message.push('\n');
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
            let mut params = HashMap::new();
            params.insert("error_message".to_string(), e.to_string());
            let error_message =
                get_message_or_key(&ctx, MessageTextId::CommonErrorPrefix, params).await;
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
