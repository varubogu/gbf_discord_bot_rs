/// auto_recruitment_match_rulesテーブル設定
use crate::models::entities;

use super::TableConfig;

/// auto_recruitment_match_rulesテーブル設定
pub struct AutoRecruitmentMatchRulesTable;

impl TableConfig for AutoRecruitmentMatchRulesTable {
    type Entity = entities::guild_master::auto_recruitment_match_rules::Entity;

    fn table_name() -> &'static str {
        "auto_recruitment_match_rules"
    }
}
