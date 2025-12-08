use crate::facades::recruitment::change::change_recruitment_information;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::datetime_parser;
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::Message;
use std::sync::Arc;

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    // context_menu_command = "recruit_change",
    slash_command,
    name_localized("ja", "マルチバトル募集内容変更"),
    description_localized("ja", "マルチバトル募集内容を変更します。")
)]
pub async fn recruit_change(
    ctx: PoiseContext<'_>,

    #[name_localized("ja", "募集メッセージ")]
    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,

    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名（変更する場合のみ指定）")]
    #[autocomplete = "quest_auto_complete"]
    quest: Option<String>,

    #[name_localized("ja", "クエスト出発日時")]
    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時（変更する場合のみ指定）")]
    event_date: Option<String>,  

    #[name_localized("ja", "マルチ攻略方法")]
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

    // タイムゾーンを取得（日時が指定されている場合のみ）
    let parsed_date = if let Some(date_str) = event_date {
        // ギルドIDを取得
        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string()))?;

        // タイムゾーンを取得
        let app_state = ctx.data();
        let timezone_repo = Arc::new(GuildTimezoneRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service
            .get_guild_timezone(app_state.app_state.guild_db(), guild_id.get() as i64)
            .await?;

        // 日時文字列をDateTime<Utc>に変換（サーバー設定のタイムゾーンとして解釈）
        Some(datetime_parser::parse_event_date(&date_str, timezone)?)
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
