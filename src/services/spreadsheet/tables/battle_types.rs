/// battle_typesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// battle_typesテーブル設定
pub struct BattleTypesTable;

impl TableConfig for BattleTypesTable {
    type Entity = entities::battle_types::Entity;

    fn table_name() -> &'static str {
        "battle_types"
    }
}
