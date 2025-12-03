use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::types::PoiseContext;
use futures::Stream;
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
pub async fn search_quests_for_autocomplete<'a>(
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
            error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    futures::stream::iter(results)
}
