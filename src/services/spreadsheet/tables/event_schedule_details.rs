/// event_schedule_detailsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// event_schedule_detailsテーブル設定
pub struct EventScheduleDetailsTable;

impl TableConfig for EventScheduleDetailsTable {
    type Entity = entities::event_schedule_details::Entity;

    fn table_name() -> &'static str {
        "event_schedule_details"
    }
}
