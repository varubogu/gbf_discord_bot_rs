use futures::Stream;
use crate::facades::battle_recruitment;
use crate::types::{BattleType, PoiseContext, PoiseError};
// use crate::services::battle_recruitment::_recruitment::RecruitmentService;

#[poise::command(
    slash_command,
    name_localized("ja", "募集"),
    description_localized("ja", "バトル募集を作成します"),
)]
pub async fn handle_recruit_command(
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

) -> Result<(), PoiseError> {
    ctx.defer().await?;

    // Use default battle_recruitment type for now
    let _battle_type = BattleType::Default;

    // let _event_datetime = RecruitmentService::parse_event_date(&event_date).await?;

    // TODO: Use quest parameter properly
    let _quest = quest;

    battle_recruitment::new(&ctx).await;
    Ok(())
}

async fn quest_auto_complete<'a>(
    _ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    // Use a static list to avoid borrowing local variables
    const QUEST_LIST: &[&str] = &["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"];
    
    // Pre-filter the list synchronously to avoid lifetime issues
    let filtered_items: Vec<String> = QUEST_LIST
        .iter()
        .filter(|name| name.starts_with(partial))
        .map(|name| name.to_string())
        .collect();
    
    futures::stream::iter(filtered_items)
}
