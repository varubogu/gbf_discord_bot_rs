use crate::errors::RecruitmentError;
use crate::events::helpers::resolve_guild_locale;
use crate::events::interactions::components::recruit_change_handler;
use crate::services::message::MessageTextId;
use crate::services::recruitment::recruit_datetime_service::RecruitDateTimeService;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ActionRowComponent, Context, CreateInteractionResponse, EditInteractionResponse,
    ModalInteraction,
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

/// 日時入力モーダルからの送信を処理
pub async fn handle_recruit_change_date_modal(
    ctx: &Context,
    interaction: &ModalInteraction,
    data: &PoiseData,
) -> Result<()> {
    // カスタムIDからチャンネルIDとメッセージIDを抽出
    let custom_id_parts: Vec<&str> = interaction.data.custom_id.split(':').collect();
    if custom_id_parts.len() != 3 || custom_id_parts[0] != "recruit_change_date_modal" {
        return Err(AppError::from(RecruitmentError::InvalidCustomId));
    }

    let channel_id: u64 = custom_id_parts[1].parse().map_err(|_| {
        AppError::from(RecruitmentError::ParseFailed {
            field: "チャンネルID",
        })
    })?;
    let message_id: u64 = custom_id_parts[2].parse().map_err(|_| {
        AppError::from(RecruitmentError::ParseFailed {
            field: "メッセージID",
        })
    })?;

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
        .ok_or_else(|| AppError::from(RecruitmentError::MissingInput { field: "日時" }))?;

    // ギルドIDを取得
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::from(RecruitmentError::GuildOnly))?
        .get();
    let locale = resolve_guild_locale(&data.app_state, Some(guild_id as i64)).await;

    // 既存メッセージを更新するため、Deferred Updateで応答する
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await?;

    // 日時文字列を共通サービスで解析
    let event_date = {
        let date_time_service =
            RecruitDateTimeService::new(data.app_state.repositories.guild_settings);
        match date_time_service
            .parse_quest_departure(data.app_state.guild_db(), guild_id as i64, &event_date_str)
            .await
        {
            Ok(datetime) => datetime,
            Err(AppError::Business { .. }) => {
                error!("日時の解析に失敗しました: 絶対日時ではありません");
                let parse_failed_message = get_message_or_fallback(
                    data,
                    guild_id,
                    MessageTextId::RecruitmentCommandChangeModalAbsoluteDatetimeRequired,
                    HashMap::new(),
                    &locale,
                    "日時の解析に失敗しました: 絶対日時で指定してください",
                )
                .await;
                interaction
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(parse_failed_message)
                            .components(vec![]),
                    )
                    .await?;
                return Ok(());
            }
            Err(e) => {
                error!(error = %e, "日時の解析に失敗しました");
                let mut params = HashMap::new();
                params.insert("error_message".to_string(), e.user_message());
                let parse_failed_message = get_message_or_fallback(
                    data,
                    guild_id,
                    MessageTextId::RecruitmentCommandChangeModalParseFailed,
                    params,
                    &locale,
                    &format!("日時の解析に失敗しました: {e}"),
                )
                .await;
                interaction
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(parse_failed_message)
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
