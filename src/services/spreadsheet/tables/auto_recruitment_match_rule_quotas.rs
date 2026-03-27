/// auto_recruitment_match_rule_quotasテーブル設定
use crate::models::entities;

use super::TableConfig;

/// auto_recruitment_match_rule_quotasテーブル設定
pub struct AutoRecruitmentMatchRuleQuotasTable;

impl TableConfig for AutoRecruitmentMatchRuleQuotasTable {
    type Entity = entities::guild_master::auto_recruitment_match_rule_quotas::Entity;

    fn table_name() -> &'static str {
        "auto_recruitment_match_rule_quotas"
    }
}
