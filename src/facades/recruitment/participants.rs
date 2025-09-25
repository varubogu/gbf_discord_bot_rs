use crate::types;
use crate::types::{DiscordOperation, DiscordOperationResult, PoiseContext};
use sea_orm::TransactionTrait;
use std::pin::Pin;
use tracing::{error, info, instrument};

/// 参加者を更新する（クロージャパターン）
#[instrument(level = "debug", skip(ctx))]
pub async fn update_participants<F>(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<()>
where
    F: FnMut(
        DiscordOperation,
    ) -> Pin<Box<dyn Future<Output = types::Result<DiscordOperationResult>> + Send>>,
{
    info!("BattleRecruitmentFacade::update_participants - 参加者を更新します");
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        // メッセージを取得
        // メッセージのリアクションとユーザーをそれぞれ取得
        // 参加者一覧メッセージを作成
        // 募集メッセージを編集して参加者一覧部分を反映

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "参加者更新が完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "参加者更新エラー");
            Err(e)
        }
    }
}
