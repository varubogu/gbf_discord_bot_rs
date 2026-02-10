//! 自動募集クエスト選択確認ハンドラ
//!
//! ユーザーの選択済みクエストを表示する

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::UserDesiredQuestRepository;
use crate::repository::quest_repository::QuestRepository;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context, EditInteractionResponse};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
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

    let app_state = &data.app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id).await?;

    let result: Result<String> = async {
        let quest_repo = app_state.repositories.user_desired_quest;
        let master_quest_repo = app_state.repositories.quest;

        // ユーザーの選択クエストを取得
        let user_quests = quest_repo.find_by_user(&txn, guild_id, user_id).await?;

        if user_quests.is_empty() {
            return Ok("📋 **あなたの選択済みクエスト**\n\n選択されているクエストはありません。\n\n※ クエストを選択するには上のメッセージから操作してください".to_string());
        }

        // クエストIDごとにグルーピング（同一クエストで複数属性を選択している場合）
        let mut quest_styles: HashMap<i32, Vec<i32>> = HashMap::new();
        for uq in &user_quests {
            quest_styles
                .entry(uq.quest_id)
                .or_default()
                .push(uq.battle_style_id);
        }

        // クエストIDリストを取得
        let quest_ids: Vec<i32> = quest_styles.keys().copied().collect();

        // クエスト情報を取得
        let all_quests = master_quest_repo.get_all(&txn).await?;
        let quest_map: HashMap<i32, String> = all_quests
            .into_iter()
            .filter(|q| quest_ids.contains(&q.id))
            .map(|q| (q.id, q.name))
            .collect();

        // メッセージを構築
        let mut lines: Vec<String> = Vec::new();
        lines.push("📋 **あなたの選択済みクエスト**\n".to_string());

        for (quest_id, styles) in &quest_styles {
            let quest_name = quest_map
                .get(quest_id)
                .map(|s| s.as_str())
                .unwrap_or("不明なクエスト");

            // 属性情報を構築
            let element_names: Vec<&str> = styles
                .iter()
                .filter_map(|&s| get_element_name(s))
                .collect();

            let line = if element_names.is_empty() {
                // 属性指定なしクエスト
                format!("🎮 {quest_name}")
            } else {
                // 6属性クエスト
                format!("🎮 {}（{}）", quest_name, element_names.join("、"))
            };

            lines.push(line);
        }

        lines.push("\n※ クエストを変更するには上のメッセージから操作してください".to_string());

        Ok(lines.join("\n"))
    }
    .await;

    match result {
        Ok(message) => {
            txn.commit().await?;

            interaction
                .edit_response(&ctx.http, EditInteractionResponse::new().content(message))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
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
