//! 自動募集マッチングルール検証サービス

use crate::errors::BusinessRuleError;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentMatchRuleQuotaRepository, AutoRecruitmentMatchRuleRepository,
};
use crate::services::auto_recruitment::match_rule::{
    MatchRuleDefinition, MatchRulePreset, is_six_element_quest, is_valid_six_element_style,
    quest_available_style_ids,
};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;

/// 自動募集マッチングルール検証サービス
pub struct AutoRecruitmentMatchRuleValidationService<RR, QR, QQ>
where
    RR: AutoRecruitmentMatchRuleRepository,
    QR: QuestRepository,
    QQ: AutoRecruitmentMatchRuleQuotaRepository,
{
    rule_repo: RR,
    quest_repo: QR,
    quota_repo: QQ,
}

impl<RR, QR, QQ> AutoRecruitmentMatchRuleValidationService<RR, QR, QQ>
where
    RR: AutoRecruitmentMatchRuleRepository,
    QR: QuestRepository,
    QQ: AutoRecruitmentMatchRuleQuotaRepository,
{
    pub fn new(rule_repo: RR, quest_repo: QR, quota_repo: QQ) -> Self {
        Self {
            rule_repo,
            quest_repo,
            quota_repo,
        }
    }

    /// ギルド内のマッチングルール全体を検証
    pub async fn validate_guild_rules(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<(), BusinessRuleError> {
        let rules = self
            .rule_repo
            .find_all_by_guild(txn, guild_id)
            .await
            .map_err(|_| {
                Self::table_definition_error(
                    "auto_recruitment_match_rules",
                    "ルール一覧の取得に失敗しました",
                )
            })?;
        let quotas = self
            .quota_repo
            .find_all_by_guild(txn, guild_id)
            .await
            .map_err(|_| {
                Self::table_definition_error(
                    "auto_recruitment_match_rule_quotas",
                    "属性人数設定の取得に失敗しました",
                )
            })?;

        let mut quota_map: HashMap<i32, Vec<_>> = HashMap::new();
        for quota in quotas {
            quota_map.entry(quota.quest_id).or_default().push(quota);
        }

        for rule in rules {
            let quest = self
                .quest_repo
                .get_by_target_id(txn, rule.quest_id)
                .await
                .map_err(|_| {
                    Self::table_definition_error(
                        "auto_recruitment_match_rules",
                        &format!("quest_id={} のクエスト取得に失敗しました", rule.quest_id),
                    )
                })?
                .ok_or_else(|| {
                    Self::table_definition_error(
                        "auto_recruitment_match_rules",
                        &format!("quest_id={} が存在しません", rule.quest_id),
                    )
                })?;

            let definition = MatchRuleDefinition::try_from_models(
                &rule,
                quota_map.remove(&rule.quest_id).unwrap_or_default(),
            )
            .map_err(|reason| {
                Self::table_definition_error("auto_recruitment_match_rules", &reason)
            })?;

            validate_rule_definition(rule.quest_id, &quest, &definition)?;
        }

        Ok(())
    }

    fn table_definition_error(table_name: &str, reason: &str) -> BusinessRuleError {
        table_definition_error(table_name, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::auto_recruitment::match_rule::MatchRuleQuota;
    use chrono::Utc;

    fn build_quest(
        id: i32,
        available_styles: &str,
        default_style_id: i32,
    ) -> crate::models::quests::Quest {
        crate::models::quests::Quest {
            id,
            name: format!("Quest{id}"),
            default_battle_style_id: default_style_id,
            recruit_count: 6,
            available_battle_style_ids: available_styles.to_string(),
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn validate_rule_definition_rejects_attribute_preset_for_non_six_element_quest() {
        let quest = build_quest(1, "0", 0);
        let definition = MatchRuleDefinition {
            preset: MatchRulePreset::OneEachElement,
            min_match_count: 6,
            required_battle_style_id: None,
            required_battle_style_count: None,
            quotas: Vec::new(),
        };

        let result = validate_rule_definition(1, &quest, &definition);

        assert!(result.is_err());
    }

    #[test]
    fn validate_rule_definition_rejects_fixed_quota_total_mismatch() {
        let quest = build_quest(1, "1,2,3,4,5,6", 1);
        let definition = MatchRuleDefinition {
            preset: MatchRulePreset::FixedElementQuota,
            min_match_count: 4,
            required_battle_style_id: None,
            required_battle_style_count: None,
            quotas: vec![
                MatchRuleQuota {
                    battle_style_id: 1,
                    required_count: 1,
                    sort_order: 10,
                },
                MatchRuleQuota {
                    battle_style_id: 2,
                    required_count: 1,
                    sort_order: 20,
                },
            ],
        };

        let result = validate_rule_definition(1, &quest, &definition);

        assert!(result.is_err());
    }
}

fn validate_rule_definition(
    quest_id: i32,
    quest: &crate::models::quests::Quest,
    definition: &MatchRuleDefinition,
) -> Result<(), BusinessRuleError> {
    if definition.min_match_count < 2 {
        return Err(table_definition_error(
            "auto_recruitment_match_rules",
            &format!("quest_id={quest_id} の min_match_count は 2 以上である必要があります"),
        ));
    }

    let is_six_element = is_six_element_quest(quest);
    let available_styles = quest_available_style_ids(quest);

    if definition.preset.is_attribute_based() && !is_six_element {
        return Err(table_definition_error(
            "auto_recruitment_match_rules",
            &format!(
                "quest_id={quest_id} は6属性クエストではないため属性系プリセットを使用できません"
            ),
        ));
    }

    match definition.preset {
        MatchRulePreset::MinMembersOnly => {}
        MatchRulePreset::OneEachElement => {
            if available_styles.len() != 6 {
                return Err(table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!("quest_id={quest_id} の利用可能属性が6件ではありません"),
                ));
            }

            if definition.min_match_count != 6 {
                return Err(table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!(
                        "quest_id={quest_id} の one_each_element は min_match_count=6 が必要です"
                    ),
                ));
            }
        }
        MatchRulePreset::SpecificElementNPlusAny => {
            let required_style = definition.required_battle_style_id.ok_or_else(|| {
                table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!("quest_id={quest_id} の required_battle_style_id が未設定です"),
                )
            })?;
            let required_count = definition.required_battle_style_count.ok_or_else(|| {
                table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!("quest_id={quest_id} の required_battle_style_count が未設定です"),
                )
            })?;

            if !is_valid_six_element_style(required_style)
                || !available_styles.contains(&required_style)
            {
                return Err(table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!(
                        "quest_id={quest_id} の required_battle_style_id={required_style} は利用できません"
                    ),
                ));
            }

            if required_count > definition.min_match_count {
                return Err(table_definition_error(
                    "auto_recruitment_match_rules",
                    &format!(
                        "quest_id={quest_id} の required_battle_style_count は min_match_count 以下である必要があります"
                    ),
                ));
            }
        }
        MatchRulePreset::FixedElementQuota => {
            if definition.quotas.is_empty() {
                return Err(table_definition_error(
                    "auto_recruitment_match_rule_quotas",
                    &format!("quest_id={quest_id} の fixed_element_quota には明細が必要です"),
                ));
            }

            let total_required = definition
                .quotas
                .iter()
                .map(|quota| quota.required_count)
                .sum::<usize>();
            if total_required != definition.min_match_count {
                return Err(table_definition_error(
                    "auto_recruitment_match_rule_quotas",
                    &format!(
                        "quest_id={quest_id} の required_count 合計 ({total_required}) は min_match_count ({}) と一致する必要があります",
                        definition.min_match_count
                    ),
                ));
            }

            for quota in &definition.quotas {
                if !is_valid_six_element_style(quota.battle_style_id)
                    || !available_styles.contains(&quota.battle_style_id)
                {
                    return Err(table_definition_error(
                        "auto_recruitment_match_rule_quotas",
                        &format!(
                            "quest_id={quest_id} の battle_style_id={} は利用できません",
                            quota.battle_style_id
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn table_definition_error(table_name: &str, reason: &str) -> BusinessRuleError {
    BusinessRuleError::TableDefinitionError {
        table_name: table_name.to_string(),
        reason: reason.to_string(),
    }
}
