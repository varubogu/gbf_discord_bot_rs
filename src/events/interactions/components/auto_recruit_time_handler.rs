//! 自動募集時間選択ハンドラ
//!
//! 日時チャンネルでの時間セレクトメニュー操作を処理する

use crate::facades::auto_recruitment;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::auto_recruitment::auto_recruitment_channel_repository::AutoRecruitmentChannelRepository;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, EditInteractionResponse,
};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use tracing::{error, info};

async fn get_message_or_fallback(
    data: &PoiseData,
    guild_id: u64,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    fallback_text: &str,
) -> String {
    data.app_state
        .message_service()
        .get_message(
            data.app_state.guild_db(),
            message_id.as_str(),
            params,
            Some(guild_id as i64),
            None,
        )
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}

/// 時間選択インタラクションを処理
///
/// Custom ID形式: `auto_time_select:{channel_id}`
/// channel_idからDBを検索してguild_id、month、dayを取得する
pub async fn handle_time_selection_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // DB操作があるため、即座にdeferして処理時間を確保
    interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
        error!(error = %e, "defer_ephemeralに失敗しました");
        AppError::Discord(Box::new(e))
    })?;

    // interactionからguild_idとchannel_idを取得
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("ギルドIDが取得できません".to_string()))?
        .get();
    let channel_id = extract_channel_id(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get();

    // DBからチャンネル情報を取得してmonth、dayを取得
    let app_state = &data.app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id as i64).await?;

    let channel_repo = app_state.repositories.auto_recruitment_channel;
    let channel_info = channel_repo
        .find_by_channel_id(&txn, guild_id as i64, channel_id as i64)
        .await?
        .ok_or_else(|| AppError::Generic("チャンネル情報が見つかりません".to_string()))?;

    let month = channel_info.month;
    let day = channel_info.day;

    txn.commit().await?;

    // 選択された値を取得
    let selected_hours = match &interaction.data.kind {
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

    if selected_hours.is_empty() {
        let message = get_message_or_fallback(
            data,
            guild_id,
            MessageTextId::AutoRecruitmentTimeSelectRequired,
            HashMap::new(),
            "時間を選択してください。",
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
        month,
        day,
        hour_count = selected_hours.len(),
        "時間選択を処理します"
    );

    // Facadeを呼び出し
    let app_state = &data.app_state;

    match auto_recruitment::handle_time_selection(
        app_state,
        guild_id,
        user_id,
        month,
        day,
        selected_hours.clone(),
    )
    .await
    {
        Ok(_result) => {
            let hours_str = selected_hours
                .iter()
                .map(|h| format!("{h}時"))
                .collect::<Vec<_>>()
                .join(", ");
            let mut params = HashMap::new();
            params.insert("month".to_string(), month.to_string());
            params.insert("day".to_string(), day.to_string());
            params.insert("hours_str".to_string(), hours_str.clone());
            let success_message = get_message_or_fallback(
                data,
                guild_id,
                MessageTextId::AutoRecruitmentTimeSelectRegistered,
                params,
                &format!(
                    "✅ {month}月{day}日の参加可能時間を登録しました。\n登録した時間: {hours_str}"
                ),
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
            if let Err(e) = auto_recruitment::check_and_notify_after_time_selection(
                guild_id,
                user_id,
                month,
                day,
                selected_hours,
            )
            .await
            {
                error!(error = %e, guild_id, user_id, "マッチングチェック登録に失敗しました");
            }
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, "時間選択の処理に失敗しました");
            let mut params = HashMap::new();
            params.insert("error_message".to_string(), e.to_string());
            let error_message = get_message_or_fallback(
                data,
                guild_id,
                MessageTextId::AutoRecruitmentOperationErrorPrefix,
                params,
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

/// カスタムIDからチャンネルIDを抽出
fn extract_channel_id(custom_id: &str) -> Result<u64> {
    // 形式: auto_time_select:{channel_id}
    custom_id
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| AppError::Generic("チャンネルIDの抽出に失敗しました".to_string()))
}
