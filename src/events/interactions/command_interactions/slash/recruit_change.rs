use crate::facades::recruitment::change::change_recruitment_information;
use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::types::{PoiseContext, Result};
use futures::Stream;
use poise::serenity_prelude::{AutocompleteChoice, Message};

#[poise::command(
    slash_command,
    name_localized("ja", "募集内容変更"),
    description_localized("ja", "マルチバトル募集内容を変更します。")
)]
pub async fn recruit_change(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,

    #[description = "recruit content"]
    #[description_localized("ja", "募集中の内容")]
    #[autocomplete = "auto_complete_recruit"]
    recruit: String,

    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    #[autocomplete = "auto_complete_quest"]
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

    change_recruitment_information(&ctx, &recruit, &quest, &event_date, battle_style_id).await
}

async fn auto_complete_quest<'a>(
    _ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    const QUEST_LIST: &[&str] = &["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"];

    let filtered_items: Vec<String> = QUEST_LIST
        .iter()
        .filter(|name| name.starts_with(partial))
        .map(|name| name.to_string())
        .collect();

    futures::stream::iter(filtered_items)
}

async fn auto_complete_recruit<'a>(
    _ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    const RECRUIT_LIST: &[&str] = &["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"];

    let filtered_items: Vec<String> = RECRUIT_LIST
        .iter()
        .filter(|name| name.starts_with(partial))
        .map(|name| name.to_string())
        .collect();

    futures::stream::iter(filtered_items)
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
