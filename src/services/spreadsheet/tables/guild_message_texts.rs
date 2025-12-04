/// guild_message_textsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_message_textsテーブル設定
pub struct GuildMessageTextsTable;

impl TableConfig for GuildMessageTextsTable {
    type Entity = entities::guild_message_texts::Entity;

    fn table_name() -> &'static str {
        "guild_message_texts"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
