/// elementsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// elementsテーブル設定
pub struct ElementsTable;

impl TableConfig for ElementsTable {
    type Entity = entities::master::elements::Entity;

    fn table_name() -> &'static str {
        "elements"
    }
}
