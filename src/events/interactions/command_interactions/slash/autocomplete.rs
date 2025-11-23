use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::types::PoiseContext;
use futures::Stream;
use poise::serenity_prelude::AutocompleteChoice;

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete<'a>(
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

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.db().clone();
    let battle_style_repository = SeaOrmBattleStyleRepository::new(db_conn);

    // すべての攻略方法を取得
    let battle_styles = battle_style_repository
        .get_all()
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "攻略方法の取得に失敗しました");
            vec![]
        });

    // AutocompleteChoiceに変換
    battle_styles
        .into_iter()
        .map(|style| AutocompleteChoice::new(style.display_name, style.id))
        .collect()
}
