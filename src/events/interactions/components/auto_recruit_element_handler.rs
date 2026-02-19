//! 自動募集属性選択ハンドラ
//!
//! 6属性クエストの属性セレクトメニュー操作を処理する

use crate::infrastructure::database::session::set_current_guild_id;
use crate::presenter::auto_recruitment_presenter::get_six_elements;
use crate::repository::auto_recruitment::UserDesiredQuestRepository;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionDataKind, Context};
use sea_orm::TransactionTrait;
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

    let app_state = &data.app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id).await?;

    let result: Result<Vec<i32>> = async {
        let quest_repo = app_state.repositories.user_desired_quest;

        // 既存の登録を全て削除
        quest_repo
            .delete_all_styles(&txn, guild_id, user_id, quest_id)
            .await?;

        // 選択された属性を登録
        for battle_style_id in &selected_battle_style_ids {
            quest_repo
                .create(&txn, guild_id, user_id, quest_id, *battle_style_id)
                .await?;
        }

        info!(
            guild_id,
            user_id,
            quest_id,
            count = selected_battle_style_ids.len(),
            "属性を登録しました"
        );

        Ok(selected_battle_style_ids.clone())
    }
    .await;

    match result {
        Ok(selected_ids) => {
            txn.commit().await?;

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
            txn.rollback().await?;
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
