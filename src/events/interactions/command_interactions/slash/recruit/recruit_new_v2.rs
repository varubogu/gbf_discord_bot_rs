use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::{PoiseContext, Result};
use std::sync::Arc;

use super::super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    slash_command,
    name_localized("ja", "マルチバトル募集2"),
    description_localized("ja", "マルチバトル募集を作成します（ボタン版）")
)]
pub async fn recruit_new_v2(
    ctx: PoiseContext<'_>,

    #[autocomplete = "quest_auto_complete"]
    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    quest: String,

    #[name_localized("ja", "クエスト出発日時")]
    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時")]
    event_date: String,

    #[autocomplete = "battle_style_auto_complete"]
    #[name_localized("ja", "マルチ攻略方法")]
    #[description = "battle style"]
    #[description_localized("ja", "マルチ攻略方法（未指定の場合はクエストのデフォルト値を使用）")]
    battle_style: Option<i32>,

    #[name_localized("ja", "解散時刻")]
    #[description = "dismissal times (comma-separated, max 3)"]
    #[description_localized("ja", "解散時刻（カンマ区切り、最大3つ。例: 1時間前, 21:00, 2日前）")]
    dismissal_times: Option<String>,
) -> Result<()> {
    ctx.defer().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    // タイムゾーンを取得（Facade経由）
    let app_state = &ctx.data().app_state;
    let timezone_facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let timezone = timezone_facade.get_timezone(guild_id.get() as i64).await?;

    // 日時文字列をDateTime<Utc>に変換（サーバー設定のタイムゾーンとして解釈）
    let options = DateTimeParseOptions::for_quest_departure(timezone);
    let results = parse_datetime(&event_date, &options)?;
    let parsed_date = match &results[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => {
            return Err(crate::types::AppError::Business {
                message: "クエスト出発日時は絶対日時で指定してください".to_string(),
            });
        }
    };

    // Facade呼び出し（メッセージ送信とDB保存）ボタン版
    let (_message_id, _reactions) = recruitment::new_recruit::new_recruitment(
        &ctx,
        &quest,
        battle_style,
        Some(parsed_date),
        true,
        dismissal_times,
    )
    .await?;

    // ボタンは既にメッセージに含まれているため、追加の処理は不要

    Ok(())
}
