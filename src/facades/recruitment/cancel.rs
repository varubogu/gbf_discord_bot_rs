use crate::infrastructure::database::container::RepositoryContainer;
use crate::services::recruitment::cancel::{
    CanCancelResult, cancel_recruitment_by_message, check_can_cancel_recruitment,
    create_cancel_notification_text, edit_original_message_as_cancelled,
    get_participants_from_reactions, send_cancel_reply_message,
};
use crate::types;
use crate::types::PoiseContext;
use poise::serenity_prelude::{ChannelId, MessageId};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// キャンセル可能可どうかの結果コード
pub enum CanRecruitmentCancelError {
    /// キャンセル可能
    Success,

    /// 既にキャンセル済み
    AlreadyCancelled,

    /// 募集メッセージは過去にあったが、削除済み
    MessageDeleted,

    /// 募集メッセージじゃない
    NotRecruitMessage,

    /// 募集がなく、メッセージもない
    NotFound,
}

/// 募集をキャンセルできるか確認
#[instrument]
pub async fn can_cancel_recruitment(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<CanRecruitmentCancelError> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let conn = app_state.db();
    let txn = conn.begin().await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
        let battle_recruitment_repo = repos.battle_recruitment();

        // DBの募集情報とDiscordメッセージの状況をチェック
        let can_cancel_result = check_can_cancel_recruitment(
            ctx.serenity_context(),
            guild_id,
            channel_id,
            message_id,
            battle_recruitment_repo,
        )
        .await?;

        // CanCancelResultをCanRecruitmentCancelErrorに変換
        let result = match can_cancel_result {
            CanCancelResult::Success => CanRecruitmentCancelError::Success,
            CanCancelResult::AlreadyCancelled => CanRecruitmentCancelError::AlreadyCancelled,
            CanCancelResult::MessageDeleted => CanRecruitmentCancelError::MessageDeleted,
            CanCancelResult::NotRecruitMessage => CanRecruitmentCancelError::NotRecruitMessage,
            CanCancelResult::NotFound => CanRecruitmentCancelError::NotFound,
        };

        Ok::<CanRecruitmentCancelError, crate::types::AppError>(result)
    }
    .await;

    match result {
        Ok(result) => {
            txn.commit().await?;
            info!(message_id = %message_id, "募集キャンセル可能");
            Ok(result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "募集キャンセルエラー");
            Err(e)
        }
    }
}

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
    let conn = app_state.db();
    let txn = conn.begin().await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
        let battle_recruitment_repo = repos.battle_recruitment();

        info!(
            "キャンセル処理開始: guild_id={}, channel_id={}, message_id={}",
            guild_id, channel_id, message_id
        );

        // 1. 募集メッセージを取得して内容を保存
        let channel_id_obj = ChannelId::from(channel_id);
        let original_message = channel_id_obj
            .message(&ctx.http(), MessageId::from(message_id))
            .await?;
        let original_content = original_message.content.clone();

        // 2. リアクションから参加者一覧を取得
        let participants = get_participants_from_reactions(ctx, channel_id, message_id).await?;

        // 3. 募集メッセージを編集してキャンセル状態を明記
        edit_original_message_as_cancelled(ctx, channel_id, message_id, &original_content).await?;

        // 4. キャンセル通知メッセージを作成
        let cancel_notification = create_cancel_notification_text(&participants).await?;

        // 5. キャンセル通知メッセージを送信
        let cancel_message_id =
            send_cancel_reply_message(ctx, channel_id, message_id, &cancel_notification).await?;

        // 6. DBから募集情報を取得し、キャンセル済み状態に更新
        let _recruitment = cancel_recruitment_by_message(
            &txn,
            battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            cancel_message_id,
        )
        .await?;

        info!("キャンセル処理完了");

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
