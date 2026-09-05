//! イベントレイヤー用ヘルパー関数
//!
//! PoiseContextを使用する処理を集約。servicesレイヤーに依存しつつ、
//! poise依存をeventsレイヤーに閉じ込める。

use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::locale_service::{DEFAULT_LOCALE, LocaleService};
use crate::services::message::{
    GuildMessageTextRepository, MessageService, MessageTextId, MessageTextRepository,
};
use crate::types::PoiseContext;
use crate::utils::datetime_display::format_datetime_with_weekday;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

/// 出発日時のギルドタイムゾーン表示フォーマット
const EVENT_DATETIME_FORMAT: &str = "%Y-%m-%d ({weekday}) %H:%M %Z";

/// 出発日時のUTC表示フォーマット（タイムゾーン取得失敗時のフォールバック）
const EVENT_DATETIME_FORMAT_UTC: &str = "%Y-%m-%d ({weekday}) %H:%M UTC";

/// 出発日時をギルドのタイムゾーンで曜日付きに整形する
///
/// タイムゾーンの取得に失敗した場合はwarnログを出してUTC表示にフォールバックする。
pub async fn format_event_datetime(
    app_state: &crate::types::AppState,
    guild_id: Option<u64>,
    event_date: DateTime<Utc>,
    locale: &str,
) -> String {
    let Some(guild_id) = guild_id else {
        return format_datetime_with_weekday(event_date, EVENT_DATETIME_FORMAT_UTC, locale);
    };

    let guild_settings_facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    match guild_settings_facade.get_timezone(guild_id as i64).await {
        Ok(timezone) => format_datetime_with_weekday(
            event_date.with_timezone(&timezone),
            EVENT_DATETIME_FORMAT,
            locale,
        ),
        Err(e) => {
            warn!(error = %e, guild_id = guild_id, "タイムゾーン取得に失敗したためUTC表示します");
            format_datetime_with_weekday(event_date, EVENT_DATETIME_FORMAT_UTC, locale)
        }
    }
}

/// PoiseContextから最適なロケールを取得
///
/// guild_settings.locale を参照し、未設定または取得失敗時は `ja` を返す。
pub async fn get_locale_from_context(ctx: &PoiseContext<'_>) -> String {
    let guild_id = get_guild_id_from_context(ctx);
    resolve_guild_locale(&ctx.data().app_state, guild_id).await
}

/// PoiseContextからギルドIDを取得
pub fn get_guild_id_from_context(ctx: &PoiseContext<'_>) -> Option<i64> {
    ctx.guild_id().map(|id| id.get() as i64)
}

/// guild_settings.locale を取得して返す
///
/// - guild_id がない場合: `ja`
/// - DB取得に失敗した場合: warnを出して `ja`
pub async fn resolve_guild_locale(
    app_state: &crate::types::AppState,
    guild_id: Option<i64>,
) -> String {
    let Some(guild_id) = guild_id else {
        return DEFAULT_LOCALE.to_string();
    };

    let locale_service = LocaleService::new(app_state.repositories.guild_settings);
    match locale_service
        .get_guild_locale(app_state.guild_db(), guild_id)
        .await
    {
        Ok(locale) => locale,
        Err(e) => {
            warn!(
                error = %e,
                guild_id = guild_id,
                "ギルドロケールの取得に失敗したため、デフォルト（ja）を使用します"
            );
            DEFAULT_LOCALE.to_string()
        }
    }
}

/// PoiseContextを使用してメッセージを取得
///
/// # 引数
/// * `ctx` - Poiseコンテキスト
/// * `message_service` - メッセージサービス
/// * `message_id` - メッセージID（`MessageTextId`）
/// * `params` - パラメータ
pub async fn get_message_from_context<G, M>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<G, M>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> Result<String, crate::errors::ServiceError>
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    let guild_id = get_guild_id_from_context(ctx);
    let locale = get_locale_from_context(ctx).await;

    message_service
        .get_message(
            ctx.data().app_state.guild_db(),
            message_id.as_str(),
            params,
            guild_id,
            Some(&locale),
        )
        .await
}

/// メッセージ取得に失敗した場合、呼び出し側で指定した文言へフォールバックする
pub async fn get_message_or_fallback_from_context<G, M>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<G, M>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    fallback_text: &str,
) -> String
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    get_message_from_context(ctx, message_service, message_id, params)
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}

/// メッセージ取得に失敗した場合、メッセージキーを返す
///
/// ユーザー向け文言の直書きフォールバックを避けるために使用する。
pub async fn get_message_or_key_from_context<G, M>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<G, M>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> String
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    get_message_from_context(ctx, message_service, message_id, params)
        .await
        .unwrap_or_else(|_| message_id.as_str().to_string())
}
