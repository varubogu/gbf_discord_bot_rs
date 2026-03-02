//! 自動募集属性選択ハンドラ
//!
//! 6属性クエストの属性セレクトメニュー操作を処理する

use crate::facades::auto_recruitment;
use crate::presenter::auto_recruitment_presenter::get_six_elements;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionDataKind, Context};
use tracing::{error, info};

/// 属性選択インタラクションを処理
///
/// Custom ID形式: `auto_quest_element:{guild_id}:{quest_id}`
///
/// メッセージは全ユーザーで共有されるため、セレクトメニューの見た目は変更しない。
/// ユーザーごとの選択状態はエフェメラル応答で通知する。
pub async fn handle_element_selection(
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

    // 選択された属性を取得
    let selected_battle_style_ids = match &interaction.data.kind {
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

    info!(
        guild_id,
        user_id,
        quest_id,
        ?selected_battle_style_ids,
        "属性選択を処理します"
    );

    let result = auto_recruitment::register_selected_elements(
        &data.app_state,
        guild_id,
        user_id,
        quest_id,
        selected_battle_style_ids,
    )
    .await;

    match result {
        Ok(selected) => {
            let selected_ids = selected.selected_battle_style_ids;

            // エフェメラル応答でユーザーに結果を通知
            let message = if selected_ids.is_empty() {
                "❌ 全ての属性を解除しました".to_string()
            } else {
                let elements = get_six_elements();
                let selected_names: Vec<String> = selected_ids
                    .iter()
                    .filter_map(|id| {
                        elements
                            .iter()
                            .find(|e| e.id == *id)
                            .map(|e| format!("{} {}", e.emoji, e.name))
                    })
                    .collect();
                format!("✅ 属性を登録しました: {}", selected_names.join(", "))
            };

            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new().content(message),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, quest_id, "属性選択処理に失敗しました");
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
    // 形式: auto_quest_element:{guild_id}:{quest_id}
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
        let result = extract_params("auto_quest_element:12345:99").unwrap();
        assert_eq!(result, (12345, 99));
    }

    #[test]
    fn extract_params_不正フォーマットで失敗() {
        let result = extract_params("auto_quest_element:12345");
        assert!(result.is_err());
    }
}
