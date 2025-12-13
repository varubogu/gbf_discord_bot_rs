use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
pub async fn search_quests_for_autocomplete(
    ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.guild_db().clone();
    let quest_repository = SeaOrmQuestRepository::new();

    // Service層を使って検索
    let search_service = QuestSearchService::new(&quest_repository);
    let results = search_service
        .search_for_autocomplete(&db_conn, partial)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    // AutocompleteChoiceに変換（display_nameを表示、quest_nameを値として使用）
    results
        .into_iter()
        .map(|item| AutocompleteChoice::new(item.display_name, item.quest_name))
        .collect()
}
