//! 自動募集マッチングルールのドメインモデル

use crate::models::entities::guild_master::{
    auto_recruitment_match_rule_quotas, auto_recruitment_match_rules,
};
use crate::models::quests::Quest;
use crate::types::BattleStyleId;

/// 6属性のスタイルID一覧
pub const SIX_ELEMENT_STYLE_IDS: [i32; 6] = [1, 2, 3, 4, 5, 6];

/// 自動募集マッチングのプリセット種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchRulePreset {
    MinMembersOnly,
    OneEachElement,
    SpecificElementNPlusAny,
    FixedElementQuota,
}

impl MatchRulePreset {
    /// 文字列からプリセット種別へ変換
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "min_members_only" => Some(Self::MinMembersOnly),
            "one_each_element" => Some(Self::OneEachElement),
            "specific_element_n_plus_any" => Some(Self::SpecificElementNPlusAny),
            "fixed_element_quota" => Some(Self::FixedElementQuota),
            _ => None,
        }
    }

    /// 属性割当を必要とするプリセットか判定
    pub fn is_attribute_based(self) -> bool {
        matches!(
            self,
            Self::OneEachElement | Self::SpecificElementNPlusAny | Self::FixedElementQuota
        )
    }
}

/// 固定属性人数設定
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchRuleQuota {
    pub battle_style_id: i32,
    pub required_count: usize,
    pub sort_order: i32,
}

/// 自動募集マッチングルールの解釈結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchRuleDefinition {
    pub preset: MatchRulePreset,
    pub min_match_count: usize,
    pub required_battle_style_id: Option<i32>,
    pub required_battle_style_count: Option<usize>,
    pub quotas: Vec<MatchRuleQuota>,
}

impl MatchRuleDefinition {
    /// Entityからドメインモデルへ変換
    pub fn try_from_models(
        rule: &auto_recruitment_match_rules::Model,
        quotas: Vec<auto_recruitment_match_rule_quotas::Model>,
    ) -> Result<Self, String> {
        let preset = MatchRulePreset::parse(&rule.preset_type)
            .ok_or_else(|| format!("未知のプリセットです: {}", rule.preset_type))?;

        let min_match_count = usize::try_from(rule.min_match_count)
            .map_err(|_| "min_match_count を usize へ変換できません".to_string())?;
        let required_battle_style_count = rule
            .required_battle_style_count
            .map(usize::try_from)
            .transpose()
            .map_err(|_| "required_battle_style_count を usize へ変換できません".to_string())?;

        let quotas = quotas
            .into_iter()
            .map(|quota| {
                Ok(MatchRuleQuota {
                    battle_style_id: quota.battle_style_id,
                    required_count: usize::try_from(quota.required_count)
                        .map_err(|_| "required_count を usize へ変換できません".to_string())?,
                    sort_order: quota.sort_order,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            preset,
            min_match_count,
            required_battle_style_id: rule.required_battle_style_id,
            required_battle_style_count,
            quotas,
        })
    }
}

/// クエストが6属性クエストか判定
pub fn is_six_element_quest(quest: &Quest) -> bool {
    BattleStyleId::is_six_elements(quest.default_battle_style_id)
}

/// クエストの利用可能な属性ID一覧を取得
pub fn quest_available_style_ids(quest: &Quest) -> Vec<i32> {
    let mut styles: Vec<i32> = quest
        .available_battle_style_ids
        .split(',')
        .filter_map(|value| value.trim().parse::<i32>().ok())
        .collect();
    styles.sort_unstable();
    styles.dedup();
    styles
}

/// 6属性IDか判定
pub fn is_valid_six_element_style(style_id: i32) -> bool {
    SIX_ELEMENT_STYLE_IDS.contains(&style_id)
}
