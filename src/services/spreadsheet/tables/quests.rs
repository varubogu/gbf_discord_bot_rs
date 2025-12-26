/// questsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// questsテーブル設定
pub struct QuestsTable;

impl TableConfig for QuestsTable {
    type Entity = entities::master::quests::Entity;

    fn table_name() -> &'static str {
        "quests"
    }
}
