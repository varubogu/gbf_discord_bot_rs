//! 自動募集マッチングサービス
//!
//! ユーザーの希望クエストと参加可能時間を照合し、
//! マッチングを検出してquest_matchingsテーブルに登録するサービス
//!
//! ## マッチングアルゴリズム
//!
//! 1. auto_recruitment_participantsとuser_desired_questsを結合
//! 2. 同一の(guild_id, quest_id, month, day, hour)でグルーピング
//! 3. 2人以上いるグループを抽出
//! 4. 6属性クエストの場合は属性被りを考慮してグループ分け
//! 5. quest_matchingsとquest_matching_usersに登録

use crate::models::entities::guild_master::{auto_recruitment_participants, user_desired_quests};
use crate::models::entities::worker::quest_matchings;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{QuestMatchingRepository, QuestMatchingUserRepository};
use crate::repository::database::auto_recruitment::{
    SeaOrmQuestMatchingRepository, SeaOrmQuestMatchingUserRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::types::Result;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// ギルドごとの既存マッチングキー（quest_id, month, day, hour, user_id）
type MatchKey = (i32, i32, i32, i32, i64);
/// ギルドID → 既存マッチングのセット
type ExistingMatchesMap = HashMap<i64, HashSet<MatchKey>>;
/// ユーザーごとの参加可能日時（month, day, hour）のリスト
type ParticipantTimesMap = HashMap<i64, HashMap<i64, Vec<(i32, i32, i32)>>>;
/// マッチング候補キー（guild_id, quest_id, month, day, hour）→ 候補リスト（user_id, battle_style_ids）
type CandidatesMap = HashMap<(i64, i32, i32, i32, i32), Vec<(i64, Vec<i32>)>>;

/// マッチング候補
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    pub guild_id: i64,
    pub quest_id: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    /// (user_id, battle_style_ids) のリスト
    /// battle_style_ids: そのユーザーがこのクエストで希望する属性のリスト
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
    /// (user_id, assigned_battle_style_id) のリスト
    /// assigned_battle_style_id: 0なら属性指定なし、None なら未確定
    pub users: Vec<(i64, Option<i32>)>,
}

/// 周期マッチング処理用サービス
///
/// 10秒間隔で実行される周期マッチング処理で使用する
pub struct PeriodicMatchingService;

impl Default for PeriodicMatchingService {
    fn default() -> Self {
        Self::new()
    }
}

impl PeriodicMatchingService {
    pub fn new() -> Self {
        Self
    }

    /// マッチング候補を検出
    ///
    /// 各ギルドのauto_recruitment_participantsとuser_desired_questsを結合し、
    /// 同じ(guild_id, quest_id, month, day, hour)でグルーピングして2人以上のグループを抽出
    pub async fn find_match_candidates(
        &self,
        txn: &DatabaseTransaction,
    ) -> Result<Vec<MatchCandidate>> {
        // 全ての参加可能時間を取得
        let participants = auto_recruitment_participants::Entity::find()
            .all(txn)
            .await?;

        // 全ての希望クエストを取得
        let desired_quests = user_desired_quests::Entity::find().all(txn).await?;

        // ギルドごとの既存マッチングユーザーを取得
        let matching_user_repo = SeaOrmQuestMatchingUserRepository::new();

        // guild_id -> (quest_id, month, day, hour, user_id) のセットを構築
        let mut existing_matches: ExistingMatchesMap = HashMap::new();

        // アクティブなマッチングを取得
        let active_matchings = quest_matchings::Entity::find()
            .filter(quest_matchings::Column::Status.eq("active"))
            .all(txn)
            .await?;

        for matching in active_matchings {
            let users = matching_user_repo
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

        // 参加者をギルド・ユーザーでグルーピング
        // guild_id -> user_id -> [(month, day, hour)]
        let mut participant_times: ParticipantTimesMap = HashMap::new();

        for p in participants {
            participant_times
                .entry(p.guild_id)
                .or_default()
                .entry(p.user_id)
                .or_default()
                .push((p.month, p.day, p.hour));
        }

        // 希望クエストをギルド・ユーザーでグルーピング
        // guild_id -> user_id -> quest_id -> [battle_style_id]
        let mut quest_prefs: HashMap<i64, HashMap<i64, HashMap<i32, Vec<i32>>>> = HashMap::new();

        for q in desired_quests {
            quest_prefs
                .entry(q.guild_id)
                .or_default()
                .entry(q.user_id)
                .or_default()
                .entry(q.quest_id)
                .or_default()
                .push(q.battle_style_id);
        }

        // マッチング候補を構築
        // (guild_id, quest_id, month, day, hour) -> [(user_id, [battle_style_id])]
        let mut candidates: CandidatesMap = HashMap::new();

        for (guild_id, users) in &participant_times {
            let existing = existing_matches.get(guild_id);

            if let Some(user_quests) = quest_prefs.get(guild_id) {
                for (user_id, times) in users {
                    if let Some(quests) = user_quests.get(user_id) {
                        for (month, day, hour) in times {
                            for (quest_id, battle_styles) in quests {
                                // 既存マッチングに含まれていないか確認
                                let is_already_matched = existing
                                    .map(|e| {
                                        e.contains(&(*quest_id, *month, *day, *hour, *user_id))
                                    })
                                    .unwrap_or(false);

                                if !is_already_matched {
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

        // 2人以上のグループを抽出
        let result: Vec<MatchCandidate> = candidates
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
            .collect();

        debug!(
            candidate_count = result.len(),
            "マッチング候補を検出しました"
        );

        Ok(result)
    }

    /// マッチング候補をグループ分け
    ///
    /// 属性被りや人数上限を考慮してグループを分割する
    pub async fn group_candidates(
        &self,
        txn: &DatabaseTransaction,
        candidates: Vec<MatchCandidate>,
    ) -> Result<Vec<MatchGroup>> {
        let quest_repo = SeaOrmQuestRepository::new();
        let mut all_groups: Vec<MatchGroup> = Vec::new();

        for candidate in candidates {
            // クエスト情報を取得して人数上限を確認
            let quest = quest_repo.get_by_target_id(txn, candidate.quest_id).await?;

            let recruit_count = quest.as_ref().map(|q| q.recruit_count).unwrap_or(6) as usize;

            // 属性情報を解析
            let is_six_element = quest
                .map(|q| {
                    q.available_battle_style_ids
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i32>().ok())
                        .count()
                        >= 6
                })
                .unwrap_or(false);

            // グループ分けアルゴリズムを適用
            let groups = self.apply_grouping_algorithm(&candidate, recruit_count, is_six_element);

            all_groups.extend(groups);
        }

        info!(
            group_count = all_groups.len(),
            "マッチンググループを作成しました"
        );

        Ok(all_groups)
    }

    /// グループ分けアルゴリズム
    ///
    /// 希望属性数の少ない人から優先的に配置する
    fn apply_grouping_algorithm(
        &self,
        candidate: &MatchCandidate,
        recruit_count: usize,
        is_six_element: bool,
    ) -> Vec<MatchGroup> {
        let mut groups: Vec<MatchGroup> = Vec::new();

        // 希望属性数の少ない順にソート
        let mut sorted_users = candidate.users.clone();
        sorted_users.sort_by_key(|(_, battle_styles)| battle_styles.len());

        for (user_id, battle_styles) in sorted_users {
            let mut placed = false;

            for group in &mut groups {
                if self.can_join_group(
                    user_id,
                    &battle_styles,
                    group,
                    recruit_count,
                    is_six_element,
                ) {
                    // グループに追加
                    let assigned_style = if is_six_element {
                        // 属性を仮割り当て（空いている属性の中から選択）
                        self.find_available_style(&battle_styles, group)
                    } else {
                        // 属性指定なしクエスト
                        Some(0)
                    };
                    group.users.push((user_id, assigned_style));
                    placed = true;
                    break;
                }
            }

            if !placed {
                // 新しいグループを作成
                let assigned_style = if is_six_element {
                    battle_styles.first().copied()
                } else {
                    Some(0)
                };
                let new_group = MatchGroup {
                    guild_id: candidate.guild_id,
                    quest_id: candidate.quest_id,
                    month: candidate.month,
                    day: candidate.day,
                    hour: candidate.hour,
                    users: vec![(user_id, assigned_style)],
                };
                groups.push(new_group);
            }
        }

        // 2人以上のグループのみ返す
        groups.into_iter().filter(|g| g.users.len() >= 2).collect()
    }

    /// グループに参加可能か判定
    fn can_join_group(
        &self,
        _user_id: i64,
        battle_styles: &[i32],
        group: &MatchGroup,
        recruit_count: usize,
        is_six_element: bool,
    ) -> bool {
        // 人数上限チェック
        if group.users.len() >= recruit_count {
            return false;
        }

        // 属性指定なしクエストは属性チェック不要
        if !is_six_element {
            return true;
        }

        // 6属性クエスト: ユーザーの希望属性のうち、まだ空いている属性があるか
        let used_styles: HashSet<i32> =
            group.users.iter().filter_map(|(_, style)| *style).collect();

        for style in battle_styles {
            if !used_styles.contains(style) {
                return true; // 空きがあれば参加可能
            }
        }

        false // 全ての希望属性が埋まっている場合は別グループ
    }

    /// グループ内で空いている属性を探す
    fn find_available_style(&self, battle_styles: &[i32], group: &MatchGroup) -> Option<i32> {
        let used_styles: HashSet<i32> =
            group.users.iter().filter_map(|(_, style)| *style).collect();

        for style in battle_styles {
            if !used_styles.contains(style) {
                return Some(*style);
            }
        }

        None
    }

    /// マッチンググループをDBに保存
    pub async fn save_match_groups(
        &self,
        txn: &DatabaseTransaction,
        groups: Vec<MatchGroup>,
    ) -> Result<Vec<quest_matchings::Model>> {
        let matching_repo = SeaOrmQuestMatchingRepository::new();
        let matching_user_repo = SeaOrmQuestMatchingUserRepository::new();
        let mut created_matchings: Vec<quest_matchings::Model> = Vec::new();

        for group in groups {
            // quest_matchingsに登録（UUIDはリポジトリ側で生成）
            let matching = matching_repo
                .create(
                    txn,
                    group.guild_id,
                    group.quest_id,
                    group.month,
                    group.day,
                    group.hour,
                )
                .await?;

            // quest_matching_usersに参加者を登録
            for (user_id, battle_style_id) in &group.users {
                matching_user_repo
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
        // 1. マッチング候補を検出
        let candidates = self.find_match_candidates(txn).await?;

        if candidates.is_empty() {
            debug!("マッチング候補がありません");
            return Ok(Vec::new());
        }

        // 2. グループ分け
        let groups = self.group_candidates(txn, candidates).await?;

        if groups.is_empty() {
            debug!("2人以上のグループがありません");
            return Ok(Vec::new());
        }

        // 3. DBに保存
        let matchings = self.save_match_groups(txn, groups).await?;

        Ok(matchings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grouping_algorithm_basic() {
        let service = PeriodicMatchingService::new();

        // 3人のユーザーが同じクエストを希望（属性指定なし）
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![(100, vec![0]), (101, vec![0]), (102, vec![0])],
        };

        let groups = service.apply_grouping_algorithm(&candidate, 6, false);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].users.len(), 3);
    }

    #[test]
    fn test_grouping_algorithm_with_elements() {
        let service = PeriodicMatchingService::new();

        // 3人のユーザーが6属性クエストを希望
        // ユーザー100: 火(1), 水(2)
        // ユーザー101: 火(1), 土(3)
        // ユーザー102: 水(2), 土(3)
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![(100, vec![1, 2]), (101, vec![1, 3]), (102, vec![2, 3])],
        };

        let groups = service.apply_grouping_algorithm(&candidate, 6, true);

        // 3人全員が1グループに入れる（属性被りなし）
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].users.len(), 3);
    }

    #[test]
    fn test_grouping_algorithm_with_conflict() {
        let service = PeriodicMatchingService::new();

        // 3人のユーザーが6属性クエストを希望（火属性のみ）
        // 全員火属性のみ希望 → 別グループになる
        let candidate = MatchCandidate {
            guild_id: 1,
            quest_id: 1,
            month: 1,
            day: 25,
            hour: 21,
            users: vec![(100, vec![1]), (101, vec![1]), (102, vec![1])],
        };

        let groups = service.apply_grouping_algorithm(&candidate, 6, true);

        // 2人以上のグループがないため、0グループ
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_grouping_algorithm_overflow() {
        let service = PeriodicMatchingService::new();

        // 8人のユーザーが同じクエストを希望（属性指定なし、上限6人）
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

        let groups = service.apply_grouping_algorithm(&candidate, 6, false);

        // 6人グループと2人グループに分かれる
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].users.len(), 6);
        assert_eq!(groups[1].users.len(), 2);
    }
}
