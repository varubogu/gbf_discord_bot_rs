/// event_schedulesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// event_schedulesテーブル設定
pub struct EventSchedulesTable;

impl TableConfig for EventSchedulesTable {
    type Entity = entities::event_schedules::Entity;

    fn table_name() -> &'static str {
        "event_schedules"
    }
}
