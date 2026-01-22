//! 自動募集クエスト選択ハンドラ
//!
//! クエスト選択チャンネルでのセレクトメニュー操作を処理する

use crate::facades::auto_recruitment;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tracing::{error, info};

/// クエスト選択インタラクションを処理
///
/// Custom ID形式: `auto_quest_select:{guild_id}`
pub async fn handle_quest_selection_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // Guild IDを抽出
    let guild_id = extract_guild_id(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get();

    // 選択された値を取得
    let selected_quest_ids = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect::<Vec<_>>(),
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ));
        }
    };

    if selected_quest_ids.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("クエストを選択してください。")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    info!(
        guild_id,
        user_id,
        quest_count = selected_quest_ids.len(),
        "クエスト選択を処理します"
    );

    // Facadeを呼び出し
    let app_state = &data.app_state;

    match auto_recruitment::handle_quest_selection(
        ctx,
        app_state,
        guild_id,
        user_id,
        selected_quest_ids.clone(),
    )
    .await
    {
        Ok(_result) => {
            let quest_count = selected_quest_ids.len();
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!(
                                "✅ {}個のクエストを登録しました。\n次に、参加可能な日時チャンネルで時間を選択してください。",
                                quest_count
                            ))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, "クエスト選択の処理に失敗しました");
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("エラー: {}", e))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

/// カスタムIDからGuild IDを抽出
fn extract_guild_id(custom_id: &str) -> Result<u64> {
    // 形式: auto_quest_select:{guild_id}
    custom_id
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| AppError::Generic("Guild IDの抽出に失敗しました".to_string()))
}
