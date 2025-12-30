/// guild_channelsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// guild_channelsテーブル設定
pub struct GuildChannelsTable;

impl TableConfig for GuildChannelsTable {
    type Entity = entities::guild_master::guild_channels::Entity;

    fn table_name() -> &'static str {
        "guild_channels"
    }

    /// ギルド版テーブルはスプレッドシートにguild_idを含まないため除外
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["guild_id", "created_at", "updated_at"]
    }
}
