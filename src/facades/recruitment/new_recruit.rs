use crate::infrastructure::database::container::RepositoryContainer;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::recruitment::new;
use crate::types;
use crate::types::PoiseContext;
use crate::types::battle_type::BattleType;
use chrono::{DateTime, Local};
use sea_orm::TransactionTrait;
use tracing::{info, instrument};

/// 新しい募集を開始する
#[instrument(level = "debug", skip(ctx))]
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
    event_date: Option<DateTime<Local>>,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let conn = app_state.db();
    let txn = conn.begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
        let battle_recruitment_repo = repos.battle_recruitment();
        let quest_repository = SeaOrmQuestRepository::new(conn.clone());

        // 1. 募集データ作成（QuestRepositoryを使用）
        let recruitment_data =
            new::create_recruitment_data(&quest_repository, quest_alias, battle_type, channel_id, guild_id, event_date)
                .await?;

        // 2. メッセージ送信
        let message_id = new::send_recruitment_message(ctx, &recruitment_data).await?;

        // 3. リアクション追加
        new::add_recruitment_reactions(ctx, message_id, &recruitment_data.reactions).await?;

        // 4. データ保存
        new::save_recruitment(&txn, battle_recruitment_repo, &recruitment_data, message_id).await?;

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
