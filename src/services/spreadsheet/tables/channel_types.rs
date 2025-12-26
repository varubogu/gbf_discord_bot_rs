/// channel_typesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// channel_typesテーブル設定
pub struct ChannelTypesTable;

impl TableConfig for ChannelTypesTable {
    type Entity = entities::master::channel_types::Entity;

    fn table_name() -> &'static str {
        "channel_types"
    }
}
