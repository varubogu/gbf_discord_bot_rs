use crate::events::interactions::components::recruit_change_handler;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::datetime_parser;
use crate::services::timezone_service::TimezoneService;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ActionRowComponent, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, ModalInteraction,
};
use std::sync::Arc;
use tracing::{error, info};

/// 日時入力モーダルからの送信を処理
pub async fn handle_recruit_change_date_modal(
    ctx: &Context,
    interaction: &ModalInteraction,
    data: &PoiseData,
) -> Result<()> {
    // カスタムIDからメッセージIDを抽出
    let custom_id_parts: Vec<&str> = interaction.data.custom_id.split(':').collect();
    if custom_id_parts.len() != 2 || custom_id_parts[0] != "recruit_change_date_modal" {
        return Err(AppError::Generic("不正なカスタムIDです".to_string()));
    }

    let message_id_str = custom_id_parts[1];
    let message_id: u64 = message_id_str
        .parse()
        .map_err(|_| AppError::Generic("メッセージIDの解析に失敗しました".to_string()))?;

    // モーダルから日時を取得
    let event_date_str = interaction
        .data
        .components
        .get(0)
        .and_then(|row| row.components.get(0))
        .and_then(|component| {
            if let ActionRowComponent::InputText(input) = component {
                input.value.as_ref().filter(|s| !s.trim().is_empty()).cloned()
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

    // Deferして処理時間を確保
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    // タイムゾーンを取得
    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    let timezone_service = TimezoneService::new(timezone_repo);
    let timezone = timezone_service
        .get_guild_timezone(data.app_state.guild_db(), guild_id as i64)
        .await?;

    // 日時文字列を解析
    let event_date = match datetime_parser::parse_event_date(&event_date_str, timezone) {
        Ok(dt) => dt,
        Err(e) => {
            error!(error = %e, "日時の解析に失敗しました");
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(format!("日時の解析に失敗しました: {}", e))
                        .components(vec![]),
                )
                .await?;
            return Ok(());
        }
    };

    info!(
        message_id = %message_id,
        event_date = %event_date,
        "出発日時が入力されました"
    );

    // 募集情報を更新
    let result = recruit_change_handler::update_recruitment_date(
        ctx,
        data,
        guild_id,
        message_id,
        event_date,
    )
    .await;

    // 結果をユーザーに通知
    match result {
        Ok(_) => {
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content("出発日時を変更しました")
                        .components(vec![]),
                )
                .await?;
            info!(message_id = %message_id, "出発日時変更が完了しました");
        }
        Err(e) => {
            error!(error = %e, message_id = %message_id, "出発日時変更に失敗しました");
            let user_message = e.user_message();
            interaction
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(user_message)
                        .components(vec![]),
                )
                .await?;
        }
    }

    Ok(())
}
