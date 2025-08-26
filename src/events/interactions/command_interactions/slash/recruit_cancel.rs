use crate::types;
use crate::types::PoiseContext;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, Message,
};
use std::time::Duration;

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

    let _guild_id = ctx.guild_id().unwrap_or_default().get();
    let _channel_id = message.channel_id.get();
    let _message_id = message.id.get();

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

    // ボタンクリックを待機（30秒間）
    let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(move |mci| {
            mci.data.custom_id.starts_with("confirm_cancel")
                || mci.data.custom_id.starts_with("deny_cancel")
        })
        .await;

    if let Some(interaction) = interaction {
        match interaction.data.custom_id.as_str() {
            "confirm_cancel" => {
                match crate::facades::recruitment::cancel::cancel_recruitment(
                    ctx,
                    _guild_id,
                    _channel_id,
                    _message_id,
                )
                .await
                {
                    Ok(_) => {
                        interaction
                            .create_response(
                                &ctx.serenity_context().http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("募集が正常にキャンセルされました。")
                                        .components(vec![]),
                                ),
                            )
                            .await?;
                    }
                    Err(e) => {
                        // エラーが発生した場合は、ボタンは残したままエラーメッセージを表示
                        interaction
                            .create_response(
                                &ctx.serenity_context().http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(format!(
                                            "キャンセル処理中にエラーが発生しました: {}",
                                            e
                                        ))
                                        .ephemeral(true),
                                ),
                            )
                            .await?;
                    }
                }
            }
            "deny_cancel" => {
                // キャンセルされた場合
                interaction
                    .create_response(
                        &ctx.serenity_context().http,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .content("キャンセル操作を中止しました。")
                                .components(vec![]),
                        ),
                    )
                    .await?;
            }
            _ => {
                // 予期しないボタンID
                interaction
                    .create_response(
                        &ctx.serenity_context().http,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .content("エラーが発生しました。")
                                .components(vec![]),
                        ),
                    )
                    .await?;
            }
        }
    } else {
        // タイムアウトした場合
        reply
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("操作がタイムアウトしました。")
                    .components(vec![]),
            )
            .await?;
    }

    Ok(())
}
