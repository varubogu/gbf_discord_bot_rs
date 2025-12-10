use crate::facades::recruitment;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::datetime_parser;
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use std::sync::Arc;

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

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
) -> Result<()> {
    ctx.defer().await?;

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
    let parsed_date = datetime_parser::parse_event_date(&event_date, timezone)?;

    // Facade呼び出し（メッセージ送信とDB保存）ボタン版
    let (_message_id, _reactions) = recruitment::new_recruit::new_recruitment(&ctx, &quest, battle_style, Some(parsed_date), true).await?;

    // ボタンは既にメッセージに含まれているため、追加の処理は不要

    Ok(())
}
