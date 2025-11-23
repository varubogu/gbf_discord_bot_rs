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
            let _ = ctx.say("この募集は既にキャンセルされています。").await;
            Ok(())
        }
        Ok(CanCancelResult::MessageDeleted) => {
            let _ = ctx.say("募集メッセージが削除されています。").await;
            Ok(())
        }
        Ok(CanCancelResult::NotRecruitMessage) => {
            let _ = ctx
                .say("指定されたメッセージは募集メッセージではありません。")
                .await;
            Ok(())
        }
        Ok(CanCancelResult::NotFound) => {
            let _ = ctx.say("指定された募集が見つかりません。").await;
            Ok(())
        }
        Err(e) => {
            // システムエラーを想定
            error!("{:?}", e);
            let _ = ctx.say("エラーが発生しました。").await;
            // エラーの種類に関わらずBotは続行
            Ok(())
        }
    }
}

// 未使用の関数を削除（ファサードで処理されるため）
