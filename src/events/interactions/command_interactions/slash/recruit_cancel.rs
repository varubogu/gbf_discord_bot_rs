use crate::facades::recruitment::cancel::{
    CanRecruitmentCancelError, can_cancel_recruitment, cancel_recruitment,
};
use crate::types;
use crate::types::{AppError, PoiseContext};
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, Http, Message,
};
use std::time::Duration;
use tracing::error;

#[poise::command(
    context_menu_command = "recruit_cancel",
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル")
)]
pub async fn cancel(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> types::Result<()> {
    ctx.defer().await?;
    let http = &ctx.http();

    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    // キャンセル可能かチェック
    let can_cancel_result = can_cancel_recruitment(ctx, guild_id, channel_id, message_id).await?;

    // チェック以前に終了するパターン
    match is_exit(ctx, can_cancel_result).await {
        Ok(_is_exit) => {
            if _is_exit {
                return Ok(());
            } else {
                // 処理続行
            }
        }
        Err(e) => {
            error!("{}", e);
            return Ok(());
        }
    }

    // キャンセル処理を続行するか確認するためのボタンを表示
    let reply = confirm_interaction(ctx).await?;

    // ボタンクリックを待機（30秒間）
    let component_interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(move |mci| {
            mci.data.custom_id.starts_with("confirm_cancel")
                || mci.data.custom_id.starts_with("deny_cancel")
        })
        .await;

    // インタラクションが無効になっていたらその時点で終了
    let interaction = match component_interaction {
        Some(interaction) => interaction,
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("操作がタイムアウトしました。")
                        .components(vec![]),
                )
                .await?;
            return Ok(());
        }
    };

    // インタラクションを待機して後で応答
    interaction.defer(http).await?;

    // キャンセル処理続行確認のボタンを押した
    match interaction.data.custom_id.as_str() {
        "confirm_cancel" => {
            let guild_id = ctx.guild_id().unwrap_or_default().get();
            let channel_id = message.channel_id.get();
            let message_id = message.id.get();

            match cancel_recruitment(ctx, guild_id, channel_id, message_id).await {
                Ok(_) => {
                    send_result_response(
                        ctx,
                        interaction,
                        "募集がキャンセルされました。".to_string(),
                    )
                    .await?;
                }
                Err(e) => {
                    send_error_response(http, &interaction, e).await?;
                }
            }
        }
        "deny_cancel" => {
            // キャンセルをキャンセルされたら確認ボタン削除
            send_result_response(ctx, interaction, "キャンセルを取り消しました。".to_string())
                .await?;
        }
        _ => {
            send_result_response(ctx, interaction, "エラーが発生しました。".to_string()).await?;
        }
    }

    Ok(())
}

/// 事前処理で終了するか判定
async fn is_exit(
    ctx: PoiseContext<'_>,
    can_cancel_result: CanRecruitmentCancelError,
) -> types::Result<bool> {
    match can_cancel_result {
        CanRecruitmentCancelError::Success => {
            // 正常な場合のみ終了せず処理に進む
            Ok(false)
        }
        CanRecruitmentCancelError::AlreadyCancelled => {
            send_cancel_response(ctx, "この募集は既にキャンセルされています。".to_string()).await?;
            Ok(true)
        }
        CanRecruitmentCancelError::MessageDeleted => {
            send_cancel_response(ctx, "募集メッセージが削除されています。".to_string()).await?;
            Ok(true)
        }
        CanRecruitmentCancelError::NotRecruitMessage => {
            send_cancel_response(
                ctx,
                "指定されたメッセージは募集メッセージではありません。".to_string(),
            )
            .await?;
            Ok(true)
        }
        CanRecruitmentCancelError::NotFound => {
            send_cancel_response(ctx, "指定された募集が見つかりません。".to_string()).await?;
            Ok(true)
        }
    }
}

async fn send_cancel_response(ctx: PoiseContext<'_>, content: String) -> types::Result<()> {
    ctx.send(
        poise::CreateReply::default()
            .content("指定された募集が見つかりません。")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// 確認メッセージ表示
async fn confirm_interaction(ctx: PoiseContext<'_>) -> types::Result<ReplyHandle> {
    // 確認メッセージとボタンを作成
    let confirm_button = CreateButton::new("confirm_cancel")
        .label("はい")
        .style(ButtonStyle::Danger);

    let cancel_button = CreateButton::new("deny_cancel")
        .label("いいえ")
        .style(ButtonStyle::Secondary);

    let action_row = CreateActionRow::Buttons(vec![confirm_button, cancel_button]);

    // 確認メッセージを送信
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content("この募集をキャンセルしますか？")
                .components(vec![action_row])
                .ephemeral(true),
        )
        .await?;

    Ok(reply)
}

/// コマンドのエラーレスポンス送信
async fn send_error_response(
    http: &&Http,
    interaction: &ComponentInteraction,
    e: AppError,
) -> Result<(), AppError> {
    // エラーが発生した場合は、ボタンは残したままエラーメッセージを表示
    interaction
        .create_response(
            http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("キャンセル処理中にエラーが発生しました: {}", e))
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

/// コマンドのレスポンスを返す送信
async fn send_result_response(
    ctx: PoiseContext<'_>,
    interaction: ComponentInteraction,
    content: String,
) -> types::Result<()> {
    interaction
        .create_response(
            &ctx.serenity_context().http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}
