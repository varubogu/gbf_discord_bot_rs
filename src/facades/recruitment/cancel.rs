use crate::infrastructure::database::container::RepositoryContainer;
use crate::services::recruitment::cancel::CancelRecruitmentService;
use crate::types;
use crate::types::PoiseContext;
use poise::serenity_prelude::{ChannelId, MessageId};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 募集をキャンセルする
#[instrument]
pub async fn cancel_recruitment(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        let repo_container = RepositoryContainer::new(app_state.db());
        let battle_recruitment_repo = repo_container.battle_recruitment();

        let cancel_service = CancelRecruitmentService::new(battle_recruitment_repo);

        // 募集メッセージを取得して内容を保存
        let channel = ChannelId::from(channel_id);
        let original_message = channel
            .message(&ctx.serenity_context().http, MessageId::from(message_id))
            .await?;
        let original_content = original_message.content.clone();

        // リアクションから参加者一覧を取得
        let participants = cancel_service
            .get_participants_from_reactions(ctx.serenity_context(), channel_id, message_id)
            .await?;

        // 募集メッセージを編集してキャンセル状態を明記
        cancel_service
            .edit_original_message_as_cancelled(
                ctx.serenity_context(),
                channel_id,
                message_id,
                &original_content,
            )
            .await?;

        // キャンセル通知メッセージを作成して送信
        let cancel_notification = cancel_service
            .create_cancel_notification_text(&participants)
            .await?;

        let cancel_message = cancel_service
            .send_cancel_reply(
                ctx.serenity_context(),
                channel_id,
                message_id,
                &cancel_notification,
            )
            .await?;

        // DBから募集情報を取得し、キャンセル済み状態に更新
        let _recruitment = cancel_service
            .cancel_by_message(guild_id, channel_id, message_id, cancel_message.id, &txn)
            .await?;

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "募集キャンセルが完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "募集キャンセルエラー");
            Err(e)
        }
    }
}
