use crate::facades::battle_recruitment;
use crate::types::{BattleType, PoiseContext, PoiseError};
use futures::Stream;
use poise::serenity_prelude::Message;

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
) -> Result<(), PoiseError> {
    ctx.defer().await?;

    let battle_type = BattleType::Default;

    // let _event_datetime = RecruitmentService::parse_event_date(&event_date).await?;

    // Call the updated battle_recruitment::new function
    match battle_recruitment::new(&ctx, &quest, battle_type).await {
        Ok(_) => {
            ctx.say("募集が正常に作成されました。").await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("募集作成に失敗しました: {}", e)).await?;
            Err(e.into())
        }
    }
}

#[poise::command(
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル")
)]
pub async fn recruit_cancel(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> Result<(), PoiseError> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();
    match battle_recruitment::cancel(&ctx, guild_id, channel_id, message_id).await {
        Ok(_) => {
            ctx.say("募集が正常に作成されました。").await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("募集作成に失敗しました: {}", e)).await?;
            Err(e.into())
        }
    }
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
