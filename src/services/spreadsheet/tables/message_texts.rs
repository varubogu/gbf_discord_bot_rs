/// message_textsテーブル設定
use crate::models::entities;

use super::TableConfig;

/// message_textsテーブル設定
pub struct MessageTextsTable;

impl TableConfig for MessageTextsTable {
    type Entity = entities::message_texts::Entity;

    fn table_name() -> &'static str {
        "message_texts"
    }

    /// スプレッドシート側で"messages"という別名を許可
    fn table_aliases() -> Vec<&'static str> {
        vec!["messages"]
    }
}
