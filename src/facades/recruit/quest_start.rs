use crate::services::battle_recruitment::start::StartRecruitmentService;
use crate::types;
use crate::types::{DiscordOperation, DiscordOperationResult, PoiseContext};
use sea_orm::TransactionTrait;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{error, info, instrument};

/// 募集を開始する（クロージャパターン）
#[instrument]
pub async fn start_recruitment<F>(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<()>
where
    F: FnMut(
            DiscordOperation,
        )
            -> Pin<Box<dyn Future<Output = types::Result<DiscordOperationResult>> + Send>>
        + std::fmt::Debug,
{
    info!("BattleRecruitmentFacade::start_recruitment - 募集を開始します");

    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        // 募集メッセージを取得
        // DBから募集情報を取得
        // DBの募集情報を更新（通知済みフラグ）
        // 募集メッセージに返信する形で「募集が更新されました」とメッセージ送信。参加者（リアクションした人）にメンション通知

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "募集開始が完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "募集開始エラー");
            Err(e)
        }
    }
}
