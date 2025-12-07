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
    #[description_localized("ja", "クエスト名またはクエスト別名（変更する場合のみ指定）")]
    #[autocomplete = "quest_auto_complete"]
    quest: Option<String>,

    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時（変更する場合のみ指定）")]
    event_date: Option<String>,

    #[description = "battle style"]
    #[description_localized("ja", "マルチ攻略方法（変更する場合のみ指定）")]
    #[autocomplete = "battle_style_auto_complete"]
    battle_style: Option<i32>,
) -> Result<()> {
    ctx.defer().await?;

    // パラメータが何も指定されていない場合はエラー
    if quest.is_none() && event_date.is_none() && battle_style.is_none() {
        return Err(crate::types::AppError::Business {
            message: "変更する項目を少なくとも1つ指定してください。".to_string(),
        });
    }

    // 日時文字列をDateTime<Utc>に変換（指定されている場合のみ）
    let parsed_date = if let Some(date_str) = event_date {
        Some(datetime_parser::parse_event_date(&date_str)?)
    } else {
        None
    };

    // 募集内容変更を実行
    change_recruitment_information(
        &ctx,
        &message,
        quest.as_deref(),
        parsed_date,
        battle_style,
    )
    .await?;

    // 処理完了をユーザーに通知
    ctx.send(
        poise::CreateReply::default()
            .content("募集内容を更新しました。")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
