use crate::facades::recruitment;
use crate::services::datetime_parser;
use crate::types::{PoiseContext, Result};

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    slash_command,
    name_localized("ja", "マルチバトル募集"),
    description_localized("ja", "マルチバトル募集を作成します")
)]
pub async fn recruit_new(
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
    battle_style: Option<i32>,
) -> Result<()> {
    ctx.defer().await?;

    // 日時文字列をDateTime<Local>に変換
    let parsed_date = datetime_parser::parse_event_date(&event_date)?;

    recruitment::new_recruit::new_recruitment(&ctx, &quest, battle_style, Some(parsed_date)).await
}
