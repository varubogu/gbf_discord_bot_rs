/// quest_aliasesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// quest_aliasesテーブル設定
pub struct QuestAliasesTable;

impl TableConfig for QuestAliasesTable {
    type Entity = entities::quest_aliases::Entity;

    fn table_name() -> &'static str {
        "quest_aliases"
    }

    /// スプレッドシート側で"quests_alias"という別名を許可
    fn table_aliases() -> Vec<&'static str> {
        vec!["quests_alias"]
    }
}
