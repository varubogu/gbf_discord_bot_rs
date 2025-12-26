/// battle_stylesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// battle_stylesテーブル設定
pub struct BattleStylesTable;

impl TableConfig for BattleStylesTable {
    type Entity = entities::master::battle_styles::Entity;

    fn table_name() -> &'static str {
        "battle_styles"
    }
}
