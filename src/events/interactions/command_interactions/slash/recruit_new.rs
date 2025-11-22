use crate::facades::recruitment;
use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::datetime_parser;
use crate::services::quest::search::QuestSearchService;
use crate::types::{PoiseContext, Result};
use futures::Stream;
use poise::serenity_prelude::AutocompleteChoice;

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
    event_date: String,

    #[description = "battle style"]
    #[description_localized("ja", "マルチ攻略方法（未指定の場合はクエストのデフォルト値を使用）")]
    #[autocomplete = "battle_style_auto_complete"]
    battle_style_id: Option<i32>,
) -> Result<()> {
    ctx.defer().await?;

    // 日時文字列をDateTime<Local>に変換
    let parsed_date = datetime_parser::parse_event_date(&event_date)?;

    recruitment::new_recruit::new_recruitment(&ctx, &quest, battle_style_id, Some(parsed_date)).await
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

async fn battle_style_auto_complete(
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
