/// environmentsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// environmentsテーブル設定
pub struct EnvironmentsTable;

impl TableConfig for EnvironmentsTable {
    type Entity = entities::master::environments::Entity;

    fn table_name() -> &'static str {
        "environments"
    }
}
