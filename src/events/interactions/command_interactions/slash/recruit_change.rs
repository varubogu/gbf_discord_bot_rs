use crate::facades::recruitment::change::change_recruitment_information;
use crate::services::datetime_parser;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::Message;

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    // context_menu_command = "recruit_change",
    slash_command,
    name_localized("ja", "マルチバトル募集内容変更"),
    description_localized("ja", "マルチバトル募集内容を変更します。")
)]
pub async fn recruit_change(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,

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
    battle_style: Option<i32>,
) -> Result<()> {
    ctx.defer().await?;

    // 日時文字列をDateTime<Utc>に変換
    let parsed_date = datetime_parser::parse_event_date(&event_date)?;

    // 募集内容変更を実行
    change_recruitment_information(
        &ctx,
        &message,
        &quest,
        parsed_date,
        battle_style,
    )
    .await
}
