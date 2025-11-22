use crate::types;
use crate::types::PoiseContext;
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 募集内容を更新する（クロージャパターン）
#[instrument(level = "debug", skip(ctx))]
pub async fn change_recruitment_information(
    ctx: &PoiseContext<'_>,
    recruit: &String,
    quest: &String,
    event_date: &String,
    battle_style_id: Option<i32>,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        // 募集メッセージを取得
        // DBから募集情報を取得
        // DBの募集情報を更新
        // discordの募集メッセージを更新
        // 募集メッセージに返信する形で「募集が更新されました」とメッセージ送信。参加者（リアクションした人）にメンション通知

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!("募集内容更新が完了しました: ");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "募集内容更新エラー");
            Err(e)
        }
    }
}
