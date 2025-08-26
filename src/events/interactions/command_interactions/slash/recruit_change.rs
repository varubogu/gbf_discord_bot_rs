use crate::facades::recruitment::change::change_recruitment_information;
use crate::types::battle_type::BattleType;
use crate::types::{PoiseContext, Result};
use futures::Stream;
use poise::serenity_prelude::Message;

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
    // #[description = "Quest Combat Style"]
    // #[description_localized("ja", "クエストの戦闘スタイル")]
    // battle_type: Option<BattleType>,
) -> Result<()> {
    ctx.defer().await?;

    let battle_type = BattleType::Default;
    change_recruitment_information(&ctx, &recruit, &quest, &event_date, &battle_type).await
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
