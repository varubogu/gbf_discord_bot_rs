use crate::models::quests::Quest;
use crate::services::recruitment::new;
use crate::types;
use crate::types::PoiseContext;
use crate::types::battle_type::BattleType;
use sea_orm::TransactionTrait;
use tracing::{info, instrument};

/// 新しい募集を開始する
#[instrument]
pub async fn quest_list(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> types::Result<Vec<Quest>> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    let result = async {
        // 1. Service層で募集データ作成
        let recruitment_data = new::create_recruitment_data(
            quest_alias,
            battle_type,
            channel_id,
            guild_id,
            app_state,
            None,
        )
        .await?;

        // 2. Service層でメッセージ送信
        let message_id = new::send_recruitment_message(ctx, &recruitment_data).await?;

        // 3. Service層でリアクション追加
        new::add_recruitment_reactions(ctx, message_id, &recruitment_data.reactions).await?;

        // 4. Service層でデータ保存
        new::save_recruitment(&recruitment_data, message_id, &txn, app_state).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(vec![])
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
