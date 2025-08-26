use crate::facades::recruit;
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
    event_date: String,
    // Temporarily removing BattleType parameter until traits are implemented
    // #[description = "Quest Combat Style"]
    // #[description_localized("ja", "クエストの戦闘スタイル")]
    // battle_type: Option<BattleType>,
) -> Result<()> {
    ctx.defer().await?;

    let battle_type = BattleType::Default;
    recruit::new_recruit::new_recruitment(&ctx, &quest, battle_type).await
}

async fn quest_auto_complete<'a>(
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
