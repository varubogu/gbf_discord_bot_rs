//! 自動募集投票ハンドラ
//!
//! マッチング通知での投票セレクトメニュー操作を処理する

use crate::facades::auto_recruitment;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tracing::{error, info};

/// 投票インタラクションを処理
///
/// Custom ID形式: `auto_vote:{matched_id}`
pub async fn handle_vote_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // Matched IDを抽出
    let matched_id = extract_matched_id(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get();
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("ギルドIDが取得できません".to_string()))?
        .get();

    // 選択された値を取得
    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .first()
            .ok_or_else(|| AppError::Generic("選択値が取得できません".to_string()))?,
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ));
        }
    };

    // "any" または クエストID
    let quest_id: Option<i32> = if selected_value == "any" {
        None
    } else {
        Some(
            selected_value
                .parse()
                .map_err(|_| AppError::Generic("クエストIDの解析に失敗しました".to_string()))?,
        )
    };

    info!(guild_id, user_id, matched_id, ?quest_id, "投票を処理します");

    // Facadeを呼び出し
    let app_state = &data.app_state;

    match auto_recruitment::handle_vote(ctx, app_state, guild_id, user_id, matched_id, quest_id)
        .await
    {
        Ok(result) => {
            let message = match result {
                auto_recruitment::voting_facade::VotingResult::Accepted => {
                    if quest_id.is_some() {
                        "✅ 投票を受け付けました。".to_string()
                    } else {
                        "✅ 「何でも良い」で投票しました。".to_string()
                    }
                }
            };

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(message)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, matched_id, "投票の処理に失敗しました");
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

/// カスタムIDからMatched IDを抽出
fn extract_matched_id(custom_id: &str) -> Result<i32> {
    // 形式: auto_vote:{matched_id}
    custom_id
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<i32>().ok())
        .ok_or_else(|| AppError::Generic("Matched IDの抽出に失敗しました".to_string()))
}
