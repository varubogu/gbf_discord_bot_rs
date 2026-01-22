//! 自動募集時間選択ハンドラ
//!
//! 日時チャンネルでの時間セレクトメニュー操作を処理する

use crate::facades::auto_recruitment;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tracing::{error, info};

/// 時間選択インタラクションを処理
///
/// Custom ID形式: `auto_time_select:{guild_id}:{month}:{day}`
pub async fn handle_time_selection_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // パラメータを抽出
    let (guild_id, month, day) = extract_params(&interaction.data.custom_id)?;
    let user_id = interaction.user.id.get();

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
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("時間を選択してください。")
                        .ephemeral(true),
                ),
            )
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
        ctx,
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
                .map(|h| format!("{}時", h))
                .collect::<Vec<_>>()
                .join(", ");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!(
                                "✅ {}月{}日の参加可能時間を登録しました。\n登録した時間: {}",
                                month, day, hours_str
                            ))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id, user_id, "時間選択の処理に失敗しました");
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

/// カスタムIDからパラメータを抽出
fn extract_params(custom_id: &str) -> Result<(u64, i32, i32)> {
    // 形式: auto_time_select:{guild_id}:{month}:{day}
    let parts: Vec<&str> = custom_id.split(':').collect();

    if parts.len() != 4 {
        return Err(AppError::Generic("カスタムIDの形式が不正です".to_string()));
    }

    let guild_id = parts[1]
        .parse::<u64>()
        .map_err(|_| AppError::Generic("Guild IDの解析に失敗しました".to_string()))?;

    let month = parts[2]
        .parse::<i32>()
        .map_err(|_| AppError::Generic("月の解析に失敗しました".to_string()))?;

    let day = parts[3]
        .parse::<i32>()
        .map_err(|_| AppError::Generic("日の解析に失敗しました".to_string()))?;

    Ok((guild_id, month, day))
}
