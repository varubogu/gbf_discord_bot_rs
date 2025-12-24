use crate::facades::recruitment;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::datetime_parser;
use crate::types::{PoiseContext, Result};
use crate::utils::discord_helper;
use poise::serenity_prelude::all::MessageId;
use std::sync::Arc;

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    slash_command,
    name_localized("ja", "マルチバトル募集"),
    description_localized("ja", "マルチバトル募集を作成します")
)]
pub async fn recruit_new(
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
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    let app_state = &ctx.data().app_state;

    // タイムゾーンを取得（Facade経由）
    let timezone_facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let timezone = timezone_facade.get_timezone(guild_id.get() as i64).await?;

    // 日時文字列をDateTime<Utc>に変換（サーバー設定のタイムゾーンとして解釈）
    let parsed_date = datetime_parser::parse_event_date(&event_date, timezone)?;

    // Facade呼び出し（メッセージ送信とDB保存）リアクション版
    let (message_id, reactions) = recruitment::new_recruit::new_recruitment(
        &ctx,
        &quest,
        battle_style,
        Some(parsed_date),
        false,
    )
    .await?;

    // リアクション追加（UI層ヘルパー経由）
    discord_helper::add_reactions(
        ctx.serenity_context(),
        ctx.channel_id(),
        MessageId::new(message_id),
        &reactions,
    )
    .await
    .map_err(crate::types::AppError::Generic)?;

    Ok(())
}
