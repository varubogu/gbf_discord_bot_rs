/// guild_environmentsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_environmentsテーブル設定
pub struct GuildEnvironmentsTable;

impl TableConfig for GuildEnvironmentsTable {
    type Entity = entities::guild_master::guild_environments::Entity;

    fn table_name() -> &'static str {
        "guild_environments"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
