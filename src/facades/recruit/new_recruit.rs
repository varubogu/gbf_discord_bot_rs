use crate::infrastructure::database::container::RepositoryContainer;
use crate::services;
use crate::types;
use crate::types::PoiseContext;
use crate::types::battle_type::BattleType;
use sea_orm::TransactionTrait;
use tracing::{info, instrument};

/// 新しい募集を開始する
#[instrument]
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    let result = async {
        // 募集データを作成
        let recruitment_data = services::battle_recruitment::new::create_recruitment_data_simple(
            quest_alias,
            battle_type,
            channel_id,
            guild_id,
        );

        // 募集メッセージ送信
        let recruit_message = ctx.say(recruitment_data.message_content).await?;
        let message = recruit_message.message().await?;

        // リアクション追加
        for reaction in &recruitment_data.reactions {
            message.react(&ctx.http(), reaction.clone()).await?;
        }

        // データベースに保存
        let repos = RepositoryContainer::new(&app_state.db_connection);
        let battle_recruitment_repo = repos.battle_recruitment();

        battle_recruitment_repo
            .create_with_txn(
                &txn,
                guild_id as i64,
                channel_id as i64,
                message.id.get() as i64,
                recruitment_data.quest.target_id,
                battle_type as i32,
                recruitment_data.expiry_date,
            )
            .await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
