use crate::events::interactions::components::recruit_change_handler;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ActionRowComponent, Context, CreateInteractionResponse, EditInteractionResponse,
    ModalInteraction,
};
use std::sync::Arc;
use tracing::{error, info};

/// 日時入力モーダルからの送信を処理
pub async fn handle_recruit_change_date_modal(
    ctx: &Context,
    interaction: &ModalInteraction,
    data: &PoiseData,
) -> Result<()> {
    // カスタムIDからチャンネルIDとメッセージIDを抽出
    let custom_id_parts: Vec<&str> = interaction.data.custom_id.split(':').collect();
    if custom_id_parts.len() != 3 || custom_id_parts[0] != "recruit_change_date_modal" {
        return Err(AppError::Generic("不正なカスタムIDです".to_string()));
    }

    let channel_id: u64 = custom_id_parts[1]
        .parse()
        .map_err(|_| AppError::Generic("チャンネルIDの解析に失敗しました".to_string()))?;
    let message_id: u64 = custom_id_parts[2]
        .parse()
        .map_err(|_| AppError::Generic("メッセージIDの解析に失敗しました".to_string()))?;

    // モーダルから日時を取得
    let event_date_str = interaction
        .data
        .components
        .first()
        .and_then(|row| row.components.first())
        .and_then(|component| {
            if let ActionRowComponent::InputText(input) = component {
                input
                    .value
                    .as_ref()
                    .filter(|s| !s.trim().is_empty())
                    .cloned()
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::Generic("日時が入力されていません".to_string()))?;

    // ギルドIDを取得
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string()))?
        .get();

    // 既存メッセージを更新するため、Deferred Updateで応答する
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await?;

    // タイムゾーンを取得
    let guild_settings_facade = GuildSettingsFacade::new(Arc::new(data.app_state.clone()));
    let timezone = guild_settings_facade.get_timezone(guild_id as i64).await?;

    // 日時文字列を解析
    let event_date = {
        let options = DateTimeParseOptions::for_quest_departure(timezone);
        match parse_datetime(&event_date_str, &options) {
            Ok(results) => match &results[0] {
                ParsedDateTime::Absolute(dt) => *dt,
                _ => {
                    error!("日時の解析に失敗しました: 絶対日時ではありません");
                    interaction
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content("日時の解析に失敗しました: 絶対日時で指定してください")
                                .components(vec![]),
                        )
                        .await?;
                    return Ok(());
                }
            },
            Err(e) => {
                error!(error = %e, "日時の解析に失敗しました");
                interaction
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(format!("日時の解析に失敗しました: {e}"))
                            .components(vec![]),
                    )
                    .await?;
                return Ok(());
            }
        }
    };

    info!(
        message_id = %message_id,
        event_date = %event_date,
        "出発日時が入力されました"
    );

    // 一時状態に保存し、パネルを再描画
    recruit_change_handler::set_event_date_draft(
        interaction.user.id.get(),
        channel_id,
        message_id,
        Some(event_date),
    )
    .await;

    let (content, components) = recruit_change_handler::build_panel_content_and_components(
        data,
        interaction.user.id.get(),
        channel_id,
        message_id,
        Some(guild_id),
    )
    .await?;

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(content)
                .components(components),
        )
        .await?;

    info!(message_id = %message_id, "出発日時を一時状態に保存しました");

    Ok(())
}
