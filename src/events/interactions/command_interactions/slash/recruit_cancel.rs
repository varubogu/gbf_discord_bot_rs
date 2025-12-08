use crate::facades::recruitment::cancel as CancelFacade;
use crate::types;
use crate::types::PoiseContext;
use crate::types::domain_interface_result::CanCancelResult;
use poise::serenity_prelude::Message;
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
            ctx.send(
                poise::CreateReply::default()
                    .content("募集は既にキャンセルされています。")
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::MessageDeleted) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("募集メッセージが削除されています。")
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotRecruitMessage) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("指定されたメッセージは募集メッセージではありません。")
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Ok(CanCancelResult::NotFound) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("指定された募集が見つかりません。")
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Err(e) => {
            // システムエラーを想定
            error!("{:?}", e);
            ctx.send(
                poise::CreateReply::default()
                    .content("エラーが発生しました。再度コマンドを実行してください。改善しない場合、開発者までお問い合わせください。")
                    .ephemeral(true),
            )
            .await?;
            // エラーの種類に関わらずBotは続行
            Ok(())
        }
    }
}
