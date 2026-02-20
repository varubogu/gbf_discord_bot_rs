//! 自動募集クエスト選択ハンドラ
//!
//! クエスト選択チャンネルでのセレクトメニュー操作を処理する

use crate::events::helpers::resolve_guild_locale;
use crate::facades::auto_recruitment;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, EditInteractionResponse,
};
use std::collections::HashMap;
use tracing::{error, info};

async fn get_message_or_fallback(
    data: &PoiseData,
    guild_id: u64,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    locale: &str,
    fallback_text: &str,
) -> String {
    data.app_state
        .message_service()
        .get_message(
            data.app_state.guild_db(),
            message_id.as_str(),
            params,
            Some(guild_id as i64),
            Some(locale),
        )
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}

/// クエスト選択インタラクションを処理
///
/// Custom ID形式: `auto_quest_select:{guild_id}`
pub async fn handle_quest_selection_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // DB操作があるため、即座にdeferして処理時間を確保
    interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
        error!(error = %e, "defer_ephemeralに失敗しました");
        AppError::Discord(Box::new(e))
    })?;

    // Guild IDを抽出
    let guild_id = extract_guild_id(&interaction.data.custom_id)?;
    let locale = resolve_guild_locale(&data.app_state, Some(guild_id as i64)).await;
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
        let message = get_message_or_fallback(
            data,
            guild_id,
            MessageTextId::AutoRecruitmentQuestSelectRequired,
            HashMap::new(),
            &locale,
            "クエストを選択してください。",
        )
        .await;
        interaction
            .edit_response(&ctx.http, EditInteractionResponse::new().content(message))
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
        app_state,
        guild_id,
        user_id,
        selected_quest_ids.clone(),
    )
    .await
    {
        Ok(_result) => {
            let quest_count = selected_quest_ids.len();
            let mut params = HashMap::new();
            params.insert("count".to_string(), quest_count.to_string());
            let success_message = get_message_or_fallback(
                data,
                guild_id,
                MessageTextId::AutoRecruitmentQuestSelectRegistered,
                params,
                &locale,
                &format!("✅ {quest_count}個のクエストを登録しました。"),
            )
            .await;

            // ユーザーへの応答を先に送信
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(success_message),
                )
                .await?;

            // マッチングは周期タスクで実行されるため、ここではログのみ
            if let Err(e) = auto_recruitment::check_and_notify_after_quest_selection(
                guild_id,
                user_id,
                selected_quest_ids,
            )
            .await
            {
                error!(error = %e, guild_id, user_id, "マッチングチェック登録に失敗しました");
            }
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, "クエスト選択の処理に失敗しました");
            let mut params = HashMap::new();
            params.insert("error_message".to_string(), e.to_string());
            let error_message = get_message_or_fallback(
                data,
                guild_id,
                MessageTextId::CommonErrorPrefix,
                params,
                &locale,
                &format!("エラー: {e}"),
            )
            .await;
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(error_message),
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
