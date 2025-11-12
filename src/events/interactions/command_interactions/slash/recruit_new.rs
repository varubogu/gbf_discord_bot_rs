use crate::facades::recruitment;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::types::battle_type::BattleType;
use crate::types::{PoiseContext, Result};
use futures::Stream;

#[poise::command(
    slash_command,
    name_localized("ja", "募集"),
    description_localized("ja", "バトル募集を作成します")
)]
pub async fn recruit(
    ctx: PoiseContext<'_>,

    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    #[autocomplete = "quest_auto_complete"]
    quest: String,

    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時")]
    _event_date: String,
    // Temporarily removing BattleType parameter until traits are implemented
    // #[description = "Quest Combat Style"]
    // #[description_localized("ja", "クエストの戦闘スタイル")]
    // battle_type: Option<BattleType>,
) -> Result<()> {
    ctx.defer().await?;

    let battle_type = BattleType::Default;
    recruitment::new_recruit::new_recruitment(&ctx, &quest, battle_type).await
}

async fn quest_auto_complete<'a>(
    ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.db().clone();
    let quest_repository = SeaOrmQuestRepository::new(db_conn);

    // Service層を使って検索
    let search_service = QuestSearchService::new(&quest_repository);
    let results = search_service
        .search_for_autocomplete(partial)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    futures::stream::iter(results)
}
