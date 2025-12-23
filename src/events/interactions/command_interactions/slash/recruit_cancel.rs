use crate::facades::recruitment::cancel as CancelFacade;
use crate::services::message::helpers::get_message_from_context;
use crate::services::message::MessageId;
use crate::types;
use crate::types::PoiseContext;
use crate::types::domain_interface_result::CanCancelResult;
use poise::serenity_prelude::Message;
use std::collections::HashMap;
use tracing::error;

#[poise::command(
    context_menu_command = "recruit_cancel",
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル")
)]
pub async fn recruit_cancel(
    ctx: PoiseContext<'_>,

    #[name_localized("ja", "募集メッセージ")]
    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> types::Result<()> {
    ctx.defer().await?;

    // キャンセル可能か確認
    match CancelFacade::can_cancel(ctx, &message).await {
        Ok(CanCancelResult::Success) => {
            // キャンセル可能な場合、確認付きでキャンセル処理を実行
            CancelFacade::confirm_cancel(ctx, &message).await
        }
        Ok(CanCancelResult::AlreadyCancelled) => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::RecruitCancelAlreadyCancelled,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "募集は既にキャンセルされています。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::MessageDeleted) => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::RecruitCancelMessageDeleted,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "募集メッセージが削除されています。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotRecruitMessage) => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::RecruitCancelInvalidMessage,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "指定されたメッセージは募集メッセージではありません。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotFound) => {
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::RecruitCancelNotFound,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "指定された募集が見つかりません。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Err(e) => {
            // システムエラーを想定
            error!("{:?}", e);
            let message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageId::RecruitCancelError,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "エラーが発生しました。再度コマンドを実行してください。改善しない場合、開発者までお問い合わせください。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
            // エラーの種類に関わらずBotは続行
            Ok(())
        }
    }
}
