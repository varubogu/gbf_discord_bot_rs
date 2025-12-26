/// guild_event_schedule_detailsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_event_schedule_detailsテーブル設定
pub struct GuildEventScheduleDetailsTable;

impl TableConfig for GuildEventScheduleDetailsTable {
    type Entity = entities::guild_master::guild_event_schedule_details::Entity;

    fn table_name() -> &'static str {
        "guild_event_schedule_details"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
