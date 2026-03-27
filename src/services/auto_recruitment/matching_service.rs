//! 自動募集マッチングサービス
//!
//! ユーザーの希望クエストと参加可能時間を照合し、
//! quest_matchings テーブルに登録するグループを構築します。

use super::match_rule::{
    MatchRuleDefinition, MatchRulePreset, is_six_element_quest, quest_available_style_ids,
};
use crate::models::entities::worker::quest_matchings;
use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentMatchRuleQuotaRepository, AutoRecruitmentMatchRuleRepository,
    AutoRecruitmentParticipantRepository, QuestMatchingRepository, QuestMatchingUserRepository,
    UserDesiredQuestRepository,
};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// ギルドごとの既存マッチングキー（quest_id, month, day, hour, user_id）
type MatchKey = (i32, i32, i32, i32, i64);
/// ギルドID → 既存マッチングのセット
type ExistingMatchesMap = HashMap<i64, HashSet<MatchKey>>;
/// ユーザーごとの参加可能日時（month, day, hour）のリスト
type ParticipantTimesMap = HashMap<i64, HashMap<i64, Vec<(i32, i32, i32)>>>;
/// マッチング候補キー → 候補リスト
type CandidatesMap = HashMap<(i64, i32, i32, i32, i32), Vec<(i64, Vec<i32>)>>;
/// ギルド・クエスト単位のルールコンテキストキャッシュ
type MatchRuleContextCache = HashMap<(i64, i32), ResolvedMatchRuleContext>;

/// マッチング候補
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    pub guild_id: i64,
    pub quest_id: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    /// `(user_id, battle_style_ids)` のリスト
    pub users: Vec<(i64, Vec<i32>)>,
}

/// マッチング結果グループ
#[derive(Debug, Clone)]
pub struct MatchGroup {
    pub guild_id: i64,
    pub quest_id: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    /// `(user_id, assigned_battle_style_id)` のリスト
    pub users: Vec<(i64, Option<i32>)>,
}

/// マッチング候補内のユーザー情報
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateUser {
    user_id: i64,
    battle_styles: Vec<i32>,
}

/// 属性割当要件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ElementRequirements {
    required_counts: [usize; 6],
    free_slots: usize,
}

impl ElementRequirements {
    fn total_slots(self) -> usize {
        self.required_counts.iter().sum::<usize>() + self.free_slots
    }
}

/// クエストごとの解決済みコンテキスト
#[derive(Debug, Clone)]
struct ResolvedMatchRuleContext {
    quest: Option<Quest>,
    rule: Option<MatchRuleDefinition>,
}

impl ResolvedMatchRuleContext {
    fn recruit_count(&self) -> usize {
        self.quest
            .as_ref()
            .map(|quest| quest.recruit_count as usize)
            .unwrap_or(6)
    }

    fn is_six_element(&self) -> bool {
        self.quest
            .as_ref()
            .map(is_six_element_quest)
            .unwrap_or(false)
    }
}

/// 周期マッチング処理用サービス
pub struct PeriodicMatchingService<PR, DR, MR, MUR, QR, RR, QQ>
where
    PR: AutoRecruitmentParticipantRepository,
    DR: UserDesiredQuestRepository,
    MR: QuestMatchingRepository,
    MUR: QuestMatchingUserRepository,
    QR: QuestRepository,
    RR: AutoRecruitmentMatchRuleRepository,
    QQ: AutoRecruitmentMatchRuleQuotaRepository,
{
    participant_repo: PR,
    desired_quest_repo: DR,
    matching_repo: MR,
    matching_user_repo: MUR,
    quest_repo: QR,
    match_rule_repo: RR,
    match_rule_quota_repo: QQ,
}

impl<PR, DR, MR, MUR, QR, RR, QQ> PeriodicMatchingService<PR, DR, MR, MUR, QR, RR, QQ>
where
    PR: AutoRecruitmentParticipantRepository,
    DR: UserDesiredQuestRepository,
    MR: QuestMatchingRepository,
    MUR: QuestMatchingUserRepository,
    QR: QuestRepository,
    RR: AutoRecruitmentMatchRuleRepository,
    QQ: AutoRecruitmentMatchRuleQuotaRepository,
{
    /// 新しいサービスインスタンスを作成
    pub fn new(
        participant_repo: PR,
        desired_quest_repo: DR,
        matching_repo: MR,
        matching_user_repo: MUR,
        quest_repo: QR,
        match_rule_repo: RR,
        match_rule_quota_repo: QQ,
    ) -> Self {
        Self {
            participant_repo,
            desired_quest_repo,
            matching_repo,
            matching_user_repo,
            quest_repo,
            match_rule_repo,
            match_rule_quota_repo,
        }
    }

    /// マッチング候補を検出
    pub async fn find_match_candidates(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<MatchCandidate>> {
        let participants = self.participant_repo.find_all(txn).await?;
        let desired_quests = self.desired_quest_repo.find_all(txn).await?;
        let existing_matches = self.collect_existing_matches(txn).await?;

        let participant_times = self.build_participant_times(participants);
        let quest_prefs = self.build_quest_preferences(desired_quests);

        let mut candidates: CandidatesMap = HashMap::new();

        for (guild_id, users) in &participant_times {
            let existing = existing_matches.get(guild_id);

            if let Some(user_quests) = quest_prefs.get(guild_id) {
                for (user_id, times) in users {
                    if let Some(quests) = user_quests.get(user_id) {
                        for (month, day, hour) in times {
                            for (quest_id, battle_styles) in quests {
                                let already_matched = existing
                                    .map(|set| {
                                        set.contains(&(*quest_id, *month, *day, *hour, *user_id))
                                    })
                                    .unwrap_or(false);

                                if !already_matched {
                                    candidates
                                        .entry((*guild_id, *quest_id, *month, *day, *hour))
                                        .or_default()
                                        .push((*user_id, battle_styles.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        let result = candidates
            .into_iter()
            .filter(|(_, users)| users.len() >= 2)
            .map(
                |((guild_id, quest_id, month, day, hour), users)| MatchCandidate {
                    guild_id,
                    quest_id,
                    month,
                    day,
                    hour,
                    users,
                },
            )
            .collect::<Vec<_>>();

        debug!(
            candidate_count = result.len(),
            "マッチング候補を検出しました"
        );
        Ok(result)
    }

    /// マッチング候補をグループ分け
    pub async fn group_candidates(
        &self,
        txn: &DatabaseTransaction,
        candidates: Vec<MatchCandidate>,
    ) -> Result<Vec<MatchGroup>> {
        let mut all_groups = Vec::new();
        let mut context_cache: MatchRuleContextCache = HashMap::new();

        for candidate in candidates {
            let context = self
                .load_match_rule_context(
                    txn,
                    candidate.guild_id,
                    candidate.quest_id,
                    &mut context_cache,
                )
                .await?;

            let groups = if let Some(rule) = context.rule.as_ref() {
                self.group_with_rule_definition(&candidate, &context, rule)?
            } else {
                self.apply_legacy_grouping(
                    &candidate,
                    context.recruit_count(),
                    context.is_six_element(),
                )
            };

            all_groups.extend(groups);
        }

        info!(
            group_count = all_groups.len(),
            "マッチンググループを作成しました"
        );
        Ok(all_groups)
    }

    /// マッチンググループをDBに保存
    pub async fn save_match_groups(
        &self,
        txn: &DatabaseTransaction,
        groups: Vec<MatchGroup>,
    ) -> Result<Vec<quest_matchings::Model>> {
        let mut created_matchings = Vec::new();

        for group in groups {
            let matching = self
                .matching_repo
                .create(
                    txn,
                    group.guild_id,
                    group.quest_id,
                    group.month,
                    group.day,
                    group.hour,
                )
                .await?;

            for (user_id, battle_style_id) in &group.users {
                self.matching_user_repo
                    .create(txn, group.guild_id, matching.id, *user_id, *battle_style_id)
                    .await?;
            }

            info!(
                matching_id = %matching.id,
                guild_id = group.guild_id,
                quest_id = group.quest_id,
                user_count = group.users.len(),
                "マッチングを作成しました"
            );
            created_matchings.push(matching);
        }

        Ok(created_matchings)
    }

    /// マッチング処理のメインエントリポイント
    pub async fn process_matching(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<quest_matchings::Model>> {
        let candidates = self.find_match_candidates(txn).await?;
        if candidates.is_empty() {
            debug!("マッチング候補がありません");
            return Ok(Vec::new());
        }

        let groups = self.group_candidates(txn, candidates).await?;
        if groups.is_empty() {
            debug!("成立するマッチンググループがありません");
            return Ok(Vec::new());
        }

        self.save_match_groups(txn, groups).await
    }

    async fn collect_existing_matches(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<ExistingMatchesMap> {
        let mut existing_matches: ExistingMatchesMap = HashMap::new();
        let active_matchings = self.matching_repo.find_all_active(txn).await?;

        for matching in active_matchings {
            let users = self
                .matching_user_repo
                .find_active_by_matching(txn, matching.guild_id, matching.id)
                .await?;
            let entry = existing_matches.entry(matching.guild_id).or_default();

            for user in users {
                entry.insert((
                    matching.quest_id,
                    matching.scheduled_month,
                    matching.scheduled_day,
                    matching.scheduled_hour,
                    user.user_id,
                ));
            }
        }

        Ok(existing_matches)
    }

    fn build_participant_times(
        &self,
        participants: Vec<
            crate::models::entities::guild_master::auto_recruitment_participants::Model,
        >,
    ) -> ParticipantTimesMap {
        let mut participant_times: ParticipantTimesMap = HashMap::new();

        for participant in participants {
            participant_times
                .entry(participant.guild_id)
                .or_default()
                .entry(participant.user_id)
                .or_default()
                .push((participant.month, participant.day, participant.hour));
        }

        participant_times
    }

    fn build_quest_preferences(
        &self,
        desired_quests: Vec<crate::models::entities::guild_master::user_desired_quests::Model>,
    ) -> HashMap<i64, HashMap<i64, HashMap<i32, Vec<i32>>>> {
        let mut quest_prefs: HashMap<i64, HashMap<i64, HashMap<i32, Vec<i32>>>> = HashMap::new();

        for desired_quest in desired_quests {
            quest_prefs
                .entry(desired_quest.guild_id)
                .or_default()
                .entry(desired_quest.user_id)
                .or_default()
                .entry(desired_quest.quest_id)
                .or_default()
                .push(desired_quest.battle_style_id);
        }

        for users in quest_prefs.values_mut() {
            for quests in users.values_mut() {
                for battle_styles in quests.values_mut() {
                    battle_styles.sort_unstable();
                    battle_styles.dedup();
                }
            }
        }

        quest_prefs
    }

    async fn load_match_rule_context(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        cache: &mut MatchRuleContextCache,
    ) -> Result<ResolvedMatchRuleContext> {
        if let Some(context) = cache.get(&(guild_id, quest_id)) {
            return Ok(context.clone());
        }

        let quest = self.quest_repo.get_by_target_id(txn, quest_id).await?;
        let rule = if quest.is_some() {
            let rule_model = self
                .match_rule_repo
                .find_by_guild_and_quest(txn, guild_id, quest_id)
                .await?;

            if let Some(rule_model) = rule_model {
                let quotas = self
                    .match_rule_quota_repo
                    .find_by_guild_and_quest(txn, guild_id, quest_id)
                    .await?;

                Some(
                    MatchRuleDefinition::try_from_models(&rule_model, quotas).map_err(|reason| {
                        AppError::Business {
                            message: format!(
                                "自動募集マッチングルールの解釈に失敗しました: guild_id={guild_id}, quest_id={quest_id}, reason={reason}"
                            ),
                        }
                    })?,
                )
            } else {
                None
            }
        } else {
            None
        };

        let context = ResolvedMatchRuleContext { quest, rule };
        cache.insert((guild_id, quest_id), context.clone());
        Ok(context)
    }

    fn group_with_rule_definition(
        &self,
        candidate: &MatchCandidate,
        context: &ResolvedMatchRuleContext,
        rule: &MatchRuleDefinition,
    ) -> Result<Vec<MatchGroup>> {
        let is_six_element = context.is_six_element();

        match rule.preset {
            MatchRulePreset::MinMembersOnly => {
                Ok(self.group_by_min_members(candidate, rule.min_match_count, is_six_element))
            }
            MatchRulePreset::OneEachElement
            | MatchRulePreset::SpecificElementNPlusAny
            | MatchRulePreset::FixedElementQuota => {
                let quest = context.quest.as_ref().ok_or_else(|| AppError::Business {
                    message: format!(
                        "属性系プリセットの処理に必要なクエスト情報がありません: quest_id={}",
                        candidate.quest_id
                    ),
                })?;
                let requirements = self.compile_element_requirements(rule, quest)?;
                Ok(self.group_by_element_requirements(candidate, requirements))
            }
        }
    }

    fn group_by_min_members(
        &self,
        candidate: &MatchCandidate,
        min_match_count: usize,
        is_six_element: bool,
    ) -> Vec<MatchGroup> {
        let mut remaining_users = self.normalize_candidate_users(&candidate.users);
        let mut groups = Vec::new();

        while remaining_users.len() >= min_match_count {
            let assigned_users = remaining_users
                .drain(0..min_match_count)
                .map(|user| {
                    (
                        user.user_id,
                        self.assign_free_style(&user.battle_styles, is_six_element),
                    )
                })
                .collect::<Vec<_>>();

            groups.push(MatchGroup {
                guild_id: candidate.guild_id,
                quest_id: candidate.quest_id,
                month: candidate.month,
                day: candidate.day,
                hour: candidate.hour,
                users: assigned_users,
            });
        }

        groups
    }

    fn compile_element_requirements(
        &self,
        rule: &MatchRuleDefinition,
        quest: &Quest,
    ) -> Result<ElementRequirements> {
        let mut requirements = ElementRequirements {
            required_counts: [0; 6],
            free_slots: 0,
        };

        match rule.preset {
            MatchRulePreset::OneEachElement => {
                for style_id in quest_available_style_ids(quest) {
                    requirements.required_counts[style_index(style_id)?] = 1;
                }
            }
            MatchRulePreset::SpecificElementNPlusAny => {
                let style_id = rule
                    .required_battle_style_id
                    .ok_or_else(|| AppError::Business {
                        message: "required_battle_style_id が未設定です".to_string(),
                    })?;
                let required_count =
                    rule.required_battle_style_count
                        .ok_or_else(|| AppError::Business {
                            message: "required_battle_style_count が未設定です".to_string(),
                        })?;
                requirements.required_counts[style_index(style_id)?] = required_count;
                requirements.free_slots = rule.min_match_count.saturating_sub(required_count);
            }
            MatchRulePreset::FixedElementQuota => {
                for quota in &rule.quotas {
                    requirements.required_counts[style_index(quota.battle_style_id)?] =
                        quota.required_count;
                }
            }
            MatchRulePreset::MinMembersOnly => {}
        }

        Ok(requirements)
    }

    fn group_by_element_requirements(
        &self,
        candidate: &MatchCandidate,
        requirements: ElementRequirements,
    ) -> Vec<MatchGroup> {
        let mut remaining_users = self.normalize_candidate_users(&candidate.users);
        let mut groups = Vec::new();

        loop {
            if remaining_users.len() < requirements.total_slots() {
                break;
            }

            let sorted_users = self.sort_users_for_requirements(&remaining_users, requirements);
            let Some(assigned_users) =
                self.find_element_group_with_backtracking(&sorted_users, requirements)
            else {
                break;
            };

            let matched_user_ids: HashSet<i64> =
                assigned_users.iter().map(|(user_id, _)| *user_id).collect();

            remaining_users.retain(|user| !matched_user_ids.contains(&user.user_id));

            groups.push(MatchGroup {
                guild_id: candidate.guild_id,
                quest_id: candidate.quest_id,
                month: candidate.month,
                day: candidate.day,
                hour: candidate.hour,
                users: assigned_users,
            });
        }

        groups
    }

    fn apply_legacy_grouping(
        &self,
        candidate: &MatchCandidate,
        recruit_count: usize,
        is_six_element: bool,
    ) -> Vec<MatchGroup> {
        let mut groups = Vec::new();
        let mut sorted_users = self.normalize_candidate_users(&candidate.users);

        for user in sorted_users.drain(..) {
            let mut placed = false;

            for group in &mut groups {
                if self.can_join_legacy_group(
                    &user.battle_styles,
                    group,
                    recruit_count,
                    is_six_element,
                ) {
                    let assigned_style = if is_six_element {
                        self.find_available_legacy_style(&user.battle_styles, group)
                    } else {
                        Some(0)
                    };
                    group.users.push((user.user_id, assigned_style));
                    placed = true;
                    break;
                }
            }

            if !placed {
                let assigned_style = if is_six_element {
                    user.battle_styles.first().copied()
                } else {
                    Some(0)
                };
                groups.push(MatchGroup {
                    guild_id: candidate.guild_id,
                    quest_id: candidate.quest_id,
                    month: candidate.month,
                    day: candidate.day,
                    hour: candidate.hour,
                    users: vec![(user.user_id, assigned_style)],
                });
            }
        }

        groups
            .into_iter()
            .filter(|group| group.users.len() >= 2)
            .collect()
    }

    fn can_join_legacy_group(
        &self,
        battle_styles: &[i32],
        group: &MatchGroup,
        recruit_count: usize,
        is_six_element: bool,
    ) -> bool {
        if group.users.len() >= recruit_count {
            return false;
        }

        if !is_six_element {
            return true;
        }

        let used_styles: HashSet<i32> =
            group.users.iter().filter_map(|(_, style)| *style).collect();
        battle_styles
            .iter()
            .any(|style| !used_styles.contains(style))
    }

    fn find_available_legacy_style(
        &self,
        battle_styles: &[i32],
        group: &MatchGroup,
    ) -> Option<i32> {
        let used_styles: HashSet<i32> =
            group.users.iter().filter_map(|(_, style)| *style).collect();
        battle_styles
            .iter()
            .find(|style| !used_styles.contains(style))
            .copied()
    }

    fn normalize_candidate_users(&self, users: &[(i64, Vec<i32>)]) -> Vec<CandidateUser> {
        let mut normalized = users
            .iter()
            .map(|(user_id, battle_styles)| {
                let mut styles = battle_styles.clone();
                styles.sort_unstable();
                styles.dedup();
                CandidateUser {
                    user_id: *user_id,
                    battle_styles: styles,
                }
            })
            .collect::<Vec<_>>();

        normalized.sort_by(|left, right| {
            left.battle_styles
                .len()
                .cmp(&right.battle_styles.len())
                .then_with(|| left.user_id.cmp(&right.user_id))
        });
        normalized
    }

    fn sort_users_for_requirements(
        &self,
        users: &[CandidateUser],
        requirements: ElementRequirements,
    ) -> Vec<CandidateUser> {
        let mut sorted = users.to_vec();

        sorted.sort_by(|left, right| {
            self.assignable_choice_count(left, requirements)
                .cmp(&self.assignable_choice_count(right, requirements))
                .then_with(|| left.user_id.cmp(&right.user_id))
        });

        sorted
    }

    fn assignable_choice_count(
        &self,
        user: &CandidateUser,
        requirements: ElementRequirements,
    ) -> usize {
        let specific_choices = user
            .battle_styles
            .iter()
            .filter_map(|style_id| style_index(*style_id).ok())
            .filter(|index| requirements.required_counts[*index] > 0)
            .count();

        if requirements.free_slots > 0 {
            specific_choices + 1
        } else if specific_choices == 0 {
            usize::MAX
        } else {
            specific_choices
        }
    }

    fn find_element_group_with_backtracking(
        &self,
        users: &[CandidateUser],
        requirements: ElementRequirements,
    ) -> Option<Vec<(i64, Option<i32>)>> {
        let mut selected_users = Vec::new();
        let mut memo = HashSet::new();

        if self.search_assignments(users, 0, requirements, &mut selected_users, &mut memo) {
            return Some(selected_users);
        }

        None
    }

    fn search_assignments(
        &self,
        users: &[CandidateUser],
        index: usize,
        requirements: ElementRequirements,
        selected_users: &mut Vec<(i64, Option<i32>)>,
        memo: &mut HashSet<(usize, [usize; 6], usize)>,
    ) -> bool {
        let remaining_slots = requirements.total_slots();
        if remaining_slots == 0 {
            return true;
        }

        if index >= users.len() || users.len() - index < remaining_slots {
            return false;
        }

        if !self.has_enough_specific_candidates(users, index, requirements) {
            return false;
        }

        let state_key = (index, requirements.required_counts, requirements.free_slots);
        if memo.contains(&state_key) {
            return false;
        }

        let user = &users[index];

        for style_id in self.assignable_specific_styles(user, requirements) {
            let mut next_requirements = requirements;
            let style_slot_index = style_id.saturating_sub(1) as usize;
            next_requirements.required_counts[style_slot_index] -= 1;
            selected_users.push((user.user_id, Some(style_id)));

            if self.search_assignments(users, index + 1, next_requirements, selected_users, memo) {
                return true;
            }

            selected_users.pop();
        }

        if requirements.free_slots > 0 {
            let mut next_requirements = requirements;
            next_requirements.free_slots -= 1;
            selected_users.push((
                user.user_id,
                self.assign_free_style(&user.battle_styles, true),
            ));

            if self.search_assignments(users, index + 1, next_requirements, selected_users, memo) {
                return true;
            }

            selected_users.pop();
        }

        if self.search_assignments(users, index + 1, requirements, selected_users, memo) {
            return true;
        }

        memo.insert(state_key);
        false
    }

    fn has_enough_specific_candidates(
        &self,
        users: &[CandidateUser],
        start_index: usize,
        requirements: ElementRequirements,
    ) -> bool {
        for (style_offset, required_count) in requirements.required_counts.iter().enumerate() {
            if *required_count == 0 {
                continue;
            }

            let style_id = (style_offset + 1) as i32;
            let candidates = users[start_index..]
                .iter()
                .filter(|user| user.battle_styles.contains(&style_id))
                .count();

            if candidates < *required_count {
                return false;
            }
        }

        true
    }

    fn assignable_specific_styles(
        &self,
        user: &CandidateUser,
        requirements: ElementRequirements,
    ) -> Vec<i32> {
        let mut styles = user
            .battle_styles
            .iter()
            .copied()
            .filter(|style_id| {
                style_index(*style_id)
                    .map(|index| requirements.required_counts[index] > 0)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        styles.sort_unstable();
        styles
    }

    fn assign_free_style(&self, battle_styles: &[i32], is_six_element: bool) -> Option<i32> {
        if is_six_element {
            battle_styles.first().copied()
        } else {
            Some(0)
        }
    }
}

fn style_index(style_id: i32) -> Result<usize> {
    if (1..=6).contains(&style_id) {
        Ok((style_id - 1) as usize)
    } else {
        Err(AppError::Business {
            message: format!("属性IDが範囲外です: {style_id}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::repositories::auto_recruitment::{
        SeaOrmAutoRecruitmentMatchRuleQuotaRepository, SeaOrmAutoRecruitmentMatchRuleRepository,
        SeaOrmAutoRecruitmentParticipantRepository, SeaOrmQuestMatchingRepository,
        SeaOrmQuestMatchingUserRepository, SeaOrmUserDesiredQuestRepository,
    };
    use crate::infrastructure::database::repositories::master_data::SeaOrmQuestRepository;
    use chrono::Utc;

    fn create_test_service() -> PeriodicMatchingService<
        SeaOrmAutoRecruitmentParticipantRepository,
        SeaOrmUserDesiredQuestRepository,
        SeaOrmQuestMatchingRepository,
        SeaOrmQuestMatchingUserRepository,
        SeaOrmQuestRepository,
        SeaOrmAutoRecruitmentMatchRuleRepository,
        SeaOrmAutoRecruitmentMatchRuleQuotaRepository,
    > {
        PeriodicMatchingService::new(
            SeaOrmAutoRecruitmentParticipantRepository::new(),
            SeaOrmUserDesiredQuestRepository::new(),
            SeaOrmQuestMatchingRepository::new(),
            SeaOrmQuestMatchingUserRepository::new(),
            SeaOrmQuestRepository::new(),
            SeaOrmAutoRecruitmentMatchRuleRepository::new(),
            SeaOrmAutoRecruitmentMatchRuleQuotaRepository::new(),
        )
    }

    fn build_quest(
        id: i32,
        recruit_count: i32,
        available_styles: &str,
        default_style_id: i32,
    ) -> Quest {
        Quest {
            id,
            name: format!("Quest{id}"),
            default_battle_style_id: default_style_id,
            recruit_count,
            available_battle_style_ids: available_styles.to_string(),
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_rule_definition(
        preset: MatchRulePreset,
        min_match_count: usize,
        required_battle_style_id: Option<i32>,
        required_battle_style_count: Option<usize>,
        quotas: Vec<(i32, usize, i32)>,
    ) -> MatchRuleDefinition {
        MatchRuleDefinition {
            preset,
            min_match_count,
            required_battle_style_id,
            required_battle_style_count,
            quotas: quotas
                .into_iter()
                .map(|(battle_style_id, required_count, sort_order)| {
                    super::super::match_rule::MatchRuleQuota {
                        battle_style_id,
                        required_count,
                        sort_order,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn test_min_members_only_splits_by_minimum_count() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![
                (100, vec![0]),
                (101, vec![0]),
                (102, vec![0]),
                (103, vec![0]),
                (104, vec![0]),
                (105, vec![0]),
                (106, vec![0]),
                (107, vec![0]),
            ],
        };
        let context = ResolvedMatchRuleContext {
            quest: Some(build_quest(1, 30, "0", 0)),
            rule: None,
        };
        let rule = build_rule_definition(MatchRulePreset::MinMembersOnly, 4, None, None, vec![]);

        let groups = service
            .group_with_rule_definition(&candidate, &context, &rule)
            .expect("grouping should succeed");

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].users.len(), 4);
        assert_eq!(groups[1].users.len(), 4);
    }

    #[test]
    fn test_one_each_element_creates_unique_assignments() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![
                (100, vec![1]),
                (101, vec![2]),
                (102, vec![3]),
                (103, vec![4]),
                (104, vec![5]),
                (105, vec![6]),
                (106, vec![1]),
            ],
        };
        let quest = build_quest(1, 6, "1,2,3,4,5,6", 1);
        let context = ResolvedMatchRuleContext {
            quest: Some(quest.clone()),
            rule: None,
        };
        let rule = build_rule_definition(MatchRulePreset::OneEachElement, 6, None, None, vec![]);

        let groups = service
            .group_with_rule_definition(&candidate, &context, &rule)
            .expect("grouping should succeed");

        assert_eq!(groups.len(), 1);
        let assigned_styles: HashSet<i32> = groups[0]
            .users
            .iter()
            .filter_map(|(_, style)| *style)
            .collect();
        assert_eq!(assigned_styles.len(), 6);
        assert!(assigned_styles.contains(&1));
        assert!(assigned_styles.contains(&6));
    }

    #[test]
    fn test_specific_element_n_plus_any_requires_specific_count() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![
                (100, vec![1]),
                (101, vec![2]),
                (102, vec![3]),
                (103, vec![4]),
            ],
        };
        let quest = build_quest(1, 6, "1,2,3,4,5,6", 1);
        let context = ResolvedMatchRuleContext {
            quest: Some(quest.clone()),
            rule: None,
        };
        let rule = build_rule_definition(
            MatchRulePreset::SpecificElementNPlusAny,
            4,
            Some(1),
            Some(2),
            vec![],
        );

        let groups = service
            .group_with_rule_definition(&candidate, &context, &rule)
            .expect("grouping should succeed");

        assert!(groups.is_empty());
    }

    #[test]
    fn test_specific_element_n_plus_any_succeeds_with_minimum_group() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![
                (100, vec![1]),
                (101, vec![1, 2]),
                (102, vec![3]),
                (103, vec![4]),
                (104, vec![5]),
            ],
        };
        let quest = build_quest(1, 6, "1,2,3,4,5,6", 1);
        let context = ResolvedMatchRuleContext {
            quest: Some(quest.clone()),
            rule: None,
        };
        let rule = build_rule_definition(
            MatchRulePreset::SpecificElementNPlusAny,
            4,
            Some(1),
            Some(2),
            vec![],
        );

        let groups = service
            .group_with_rule_definition(&candidate, &context, &rule)
            .expect("grouping should succeed");

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].users.len(), 4);
        let fire_count = groups[0]
            .users
            .iter()
            .filter(|(_, style)| *style == Some(1))
            .count();
        assert_eq!(fire_count, 2);
    }

    #[test]
    fn test_fixed_element_quota_uses_backtracking() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![(100, vec![1, 2]), (101, vec![1, 3]), (102, vec![3])],
        };
        let quest = build_quest(1, 6, "1,2,3,4,5,6", 1);
        let context = ResolvedMatchRuleContext {
            quest: Some(quest.clone()),
            rule: None,
        };
        let rule = build_rule_definition(
            MatchRulePreset::FixedElementQuota,
            3,
            None,
            None,
            vec![(1, 1, 10), (2, 1, 20), (3, 1, 30)],
        );

        let groups = service
            .group_with_rule_definition(&candidate, &context, &rule)
            .expect("grouping should succeed");

        assert_eq!(groups.len(), 1);
        let assigned = groups[0]
            .users
            .iter()
            .map(|(user_id, style)| (*user_id, style.expect("style should exist")))
            .collect::<HashMap<_, _>>();
        assert_eq!(assigned.get(&100), Some(&2));
        assert_eq!(assigned.get(&101), Some(&1));
        assert_eq!(assigned.get(&102), Some(&3));
    }

    #[test]
    fn test_legacy_grouping_with_conflict_keeps_no_group() {
        let service = create_test_service();
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![(100, vec![1]), (101, vec![1]), (102, vec![1])],
        };

        let groups = service.apply_legacy_grouping(&candidate, 6, true);
        assert!(groups.is_empty());
    }
}
