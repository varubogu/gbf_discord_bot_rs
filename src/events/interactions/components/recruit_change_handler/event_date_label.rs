//! パネル表示用の出発日時ラベル整形。

use crate::events::helpers::format_event_datetime;
use crate::services::message::MessageTextId;
use crate::types::PoiseData;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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

    format_event_datetime(&data.app_state, guild_id, event_date, locale).await
}
