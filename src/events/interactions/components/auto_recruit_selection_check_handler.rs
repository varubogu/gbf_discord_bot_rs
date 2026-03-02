//! 自動募集クエスト選択確認ハンドラ
//!
//! ユーザーの選択済みクエストを表示する

use crate::facades::auto_recruitment;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context, EditInteractionResponse};
use tracing::{error, info};

/// 属性IDと名前のマッピング
fn get_element_name(battle_style_id: i32) -> Option<&'static str> {
    match battle_style_id {
        1 => Some("火属性"),
        2 => Some("水属性"),
        3 => Some("土属性"),
        4 => Some("風属性"),
        5 => Some("光属性"),
        6 => Some("闇属性"),
        _ => None,
    }
}

/// クエスト選択確認ボタンを処理
///
/// Custom ID形式: `auto_quest_selection_check:{guild_id}`
///
/// ユーザーが選択したクエストをエフェメラルメッセージで表示する
pub async fn handle_selection_check_button(
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
    let guild_id = extract_guild_id(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get() as i64;

    info!(guild_id, user_id, "クエスト選択確認ボタンを処理します");

    let result: Result<String> = async {
        let selected_quests = auto_recruitment::get_selected_quests(&data.app_state, guild_id, user_id)
            .await?;
        if selected_quests.is_empty() {
            return Ok("📋 **あなたの選択済みクエスト**\n\n選択されているクエストはありません。\n\n※ クエストを選択するには上のメッセージから操作してください".to_string());
        }

        let mut lines: Vec<String> = Vec::new();
        lines.push("📋 **あなたの選択済みクエスト**\n".to_string());

        for selected in selected_quests {
            let element_names: Vec<&str> = selected
                .battle_style_ids
                .iter()
                .filter_map(|&style| get_element_name(style))
                .collect();

            let line = if element_names.is_empty() {
                format!("🎮 {}", selected.quest_name)
            } else {
                format!("🎮 {}（{}）", selected.quest_name, element_names.join("、"))
            };

            lines.push(line);
        }

        lines.push("\n※ クエストを変更するには上のメッセージから操作してください".to_string());
        Ok(lines.join("\n"))
    }.await;

    match result {
        Ok(message) => {
            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content(message))
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, "クエスト選択確認に失敗しました");
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(format!("エラー: {e}")),
                )
                .await?;
        }
    }

    Ok(())
}

/// カスタムIDからギルドIDを抽出
fn extract_guild_id(custom_id: &str) -> Result<i64> {
    // 形式: auto_quest_selection_check:{guild_id}
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() < 2 {
        return Err(AppError::Generic("不正なカスタムIDです".to_string()));
    }

    let guild_id = parts[1]
        .parse::<i64>()
        .map_err(|_| AppError::Generic("Guild IDの解析に失敗しました".to_string()))?;

    Ok(guild_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_guild_id_正常系() {
        let guild_id = extract_guild_id("auto_quest_selection_check:24680").unwrap();
        assert_eq!(guild_id, 24680);
    }

    #[test]
    fn extract_guild_id_不正フォーマットで失敗() {
        let result = extract_guild_id("auto_quest_selection_check");
        assert!(result.is_err());
    }
}
