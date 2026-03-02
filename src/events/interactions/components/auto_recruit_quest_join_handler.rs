//! 自動募集クエスト参加ボタンハンドラ
//!
//! 属性指定なしクエストの参加ボタン操作を処理する

use crate::facades::auto_recruitment;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use tracing::{error, info};

/// クエスト参加ボタンを処理
///
/// Custom ID形式: `auto_quest_join:{guild_id}:{quest_id}`
///
/// メッセージは全ユーザーで共有されるため、ボタンの見た目は変更しない。
/// ユーザーごとの参加状態はエフェメラル応答で通知する。
pub async fn handle_quest_join_button(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // 即座にdeferして処理時間を確保
    interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
        error!(error = %e, "defer_ephemeralに失敗しました");
        AppError::Discord(Box::new(e))
    })?;

    // パラメータを抽出
    let (guild_id, quest_id) = extract_params(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get() as i64;

    info!(
        guild_id,
        user_id, quest_id, "クエスト参加ボタンを処理します"
    );

    let result =
        auto_recruitment::toggle_quest_join(&data.app_state, guild_id, user_id, quest_id).await;

    match result {
        Ok(result) => {
            let is_now_participating = result.is_now_participating;
            info!(
                guild_id,
                user_id, quest_id, is_now_participating, "クエスト参加状態を更新しました"
            );

            // エフェメラル応答でユーザーに結果を通知
            let message = if is_now_participating {
                "✅ 参加登録しました"
            } else {
                "❌ 参加を解除しました"
            };

            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new().content(message),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, quest_id, "クエスト参加処理に失敗しました");
            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("エラー: {e}")),
                )
                .await?;
        }
    }

    Ok(())
}

/// カスタムIDからパラメータを抽出
fn extract_params(custom_id: &str) -> Result<(i64, i32)> {
    // 形式: auto_quest_join:{guild_id}:{quest_id}
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() < 3 {
        return Err(AppError::Generic("不正なカスタムIDです".to_string()));
    }

    let guild_id = parts[1]
        .parse::<i64>()
        .map_err(|_| AppError::Generic("Guild IDの解析に失敗しました".to_string()))?;
    let quest_id = parts[2]
        .parse::<i32>()
        .map_err(|_| AppError::Generic("Quest IDの解析に失敗しました".to_string()))?;

    Ok((guild_id, quest_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_params_正常系() {
        let result = extract_params("auto_quest_join:67890:77").unwrap();
        assert_eq!(result, (67890, 77));
    }

    #[test]
    fn extract_params_不正フォーマットで失敗() {
        let result = extract_params("auto_quest_join:67890");
        assert!(result.is_err());
    }
}
