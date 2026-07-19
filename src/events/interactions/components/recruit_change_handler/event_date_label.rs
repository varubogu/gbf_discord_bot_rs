//! パネル表示用の出発日時ラベル整形。

use crate::facades::guild_settings::GuildSettingsFacade;
use crate::services::message::MessageTextId;
use crate::types::PoiseData;
use crate::utils::datetime_display::format_datetime_with_weekday;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

use super::get_message_or_fallback;

/// ギルドのタイムゾーンを考慮して出発日時の表示ラベルを組み立てる
pub(super) async fn format_event_date_label(
    data: &PoiseData,
    guild_id: Option<u64>,
    event_date: Option<DateTime<Utc>>,
    locale: &str,
) -> String {
    let Some(event_date) = event_date else {
        return get_message_or_fallback(
            data,
            guild_id,
            MessageTextId::RecruitmentCommandChangePanelUnchanged,
            HashMap::new(),
            locale,
            "変更しない",
        )
        .await;
    };

    if let Some(guild_id) = guild_id {
        let timezone_facade = GuildSettingsFacade::new(Arc::new(data.app_state.clone()));
        match timezone_facade.get_timezone(guild_id as i64).await {
            Ok(timezone) => {
                let local_datetime = event_date.with_timezone(&timezone);
                return format_datetime_with_weekday(
                    local_datetime,
                    "%Y-%m-%d ({weekday}) %H:%M %Z",
                    locale,
                );
            }
            Err(e) => {
                warn!(error = %e, guild_id = guild_id, "タイムゾーン取得に失敗したためUTC表示します");
            }
        }
    }

    format_datetime_with_weekday(event_date, "%Y-%m-%d ({weekday}) %H:%M UTC", locale)
}
