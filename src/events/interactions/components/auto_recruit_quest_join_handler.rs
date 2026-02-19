//! 自動募集クエスト参加ボタンハンドラ
//!
//! 属性指定なしクエストの参加ボタン操作を処理する

use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::auto_recruitment::UserDesiredQuestRepository;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use sea_orm::TransactionTrait;
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

    let app_state = &data.app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id).await?;

    let result: Result<bool> = async {
        let quest_repo = app_state.repositories.user_desired_quest;

        // 現在の登録状況を確認
        let existing = quest_repo
            .find_by_user(&txn, guild_id, user_id)
            .await?
            .into_iter()
            .filter(|q| q.quest_id == quest_id)
            .collect::<Vec<_>>();

        let is_participating = !existing.is_empty();

        if is_participating {
            // 登録解除
            quest_repo
                .delete_all_styles(&txn, guild_id, user_id, quest_id)
                .await?;
            info!(guild_id, user_id, quest_id, "クエスト参加を解除しました");
        } else {
            // 登録（属性指定なしなのでbattle_style_id=0）
            quest_repo
                .create(&txn, guild_id, user_id, quest_id, 0)
                .await?;
            info!(guild_id, user_id, quest_id, "クエスト参加を登録しました");
        }

        Ok(!is_participating)
    }
    .await;

    match result {
        Ok(is_now_participating) => {
            txn.commit().await?;

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
            txn.rollback().await?;
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
