/// guild_event_schedulesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_event_schedulesテーブル設定
pub struct GuildEventSchedulesTable;

impl TableConfig for GuildEventSchedulesTable {
    type Entity = entities::guild_event_schedules::Entity;

    fn table_name() -> &'static str {
        "guild_event_schedules"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
