/// guild_quest_disablesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_quest_disablesテーブル設定
pub struct GuildQuestDisablesTable;

impl TableConfig for GuildQuestDisablesTable {
    type Entity = entities::guild_master::guild_quest_disables::Entity;

    fn table_name() -> &'static str {
        "guild_quest_disables"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
