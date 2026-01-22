//! 自動募集マッチングサービス
//!
//! ユーザーの希望クエストと参加可能時間を照合し、
//! マッチングを検出するサービス

use crate::repository::auto_recruitment::{
    AutoRecruitmentParticipantRepository, MatchedRecruitmentChannelRepository,
    UserDesiredQuestRepository,
};
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info};

/// マッチング結果
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// マッチングした日時
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    /// マッチングしたユーザーID一覧
    pub user_ids: Vec<i64>,
    /// 共通のクエストID一覧
    pub common_quest_ids: Vec<i32>,
    /// 既存のマッチング済み募集があるかどうか
    pub existing_matched_id: Option<i32>,
}

/// 自動募集マッチングサービス
pub struct AutoMatchingService<P, Q, M>
where
    P: AutoRecruitmentParticipantRepository,
    Q: UserDesiredQuestRepository,
    M: MatchedRecruitmentChannelRepository,
{
    participant_repo: Arc<P>,
    user_quest_repo: Arc<Q>,
    matched_repo: Arc<M>,
}

impl<P, Q, M> AutoMatchingService<P, Q, M>
where
    P: AutoRecruitmentParticipantRepository,
    Q: UserDesiredQuestRepository,
    M: MatchedRecruitmentChannelRepository,
{
    pub fn new(participant_repo: Arc<P>, user_quest_repo: Arc<Q>, matched_repo: Arc<M>) -> Self {
        Self {
            participant_repo,
            user_quest_repo,
            matched_repo,
        }
    }

    /// 参加時間追加時にマッチングを実行
    ///
    /// 指定された日時に参加登録しているユーザーを検索し、
    /// 共通するクエストがあるかチェックする
    pub async fn check_match_by_time(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<MatchResult>> {
        debug!(
            guild_id,
            user_id, month, day, hour, "参加時間追加によるマッチングをチェックします"
        );

        // 同じ日時に参加登録しているユーザーを取得
        let participants = self
            .participant_repo
            .find_users_by_datetime(txn, guild_id, month, day, hour)
            .await?;

        // 自分を除いた他のユーザーを取得
        let other_user_ids: Vec<i64> = participants
            .iter()
            .filter(|p| p.user_id != user_id)
            .map(|p| p.user_id)
            .collect();

        if other_user_ids.is_empty() {
            debug!(guild_id, user_id, month, day, hour, "他の参加者がいません");
            return Ok(None);
        }

        // 自分の希望クエストを取得
        let my_quests = self
            .user_quest_repo
            .find_by_user(txn, guild_id, user_id)
            .await?;
        let my_quest_ids: HashSet<i32> = my_quests.iter().map(|q| q.quest_id).collect();

        if my_quest_ids.is_empty() {
            debug!(guild_id, user_id, "希望クエストが登録されていません");
            return Ok(None);
        }

        // 他のユーザーの希望クエストを取得し、共通クエストを探す
        let mut matched_user_ids = vec![user_id];
        let mut common_quest_ids: HashSet<i32> = my_quest_ids.clone();

        for other_user_id in other_user_ids {
            let other_quests = self
                .user_quest_repo
                .find_by_user(txn, guild_id, other_user_id)
                .await?;
            let other_quest_ids: HashSet<i32> = other_quests.iter().map(|q| q.quest_id).collect();

            // 共通クエストがあるかチェック
            let intersection: HashSet<i32> = common_quest_ids
                .intersection(&other_quest_ids)
                .copied()
                .collect();

            if !intersection.is_empty() {
                matched_user_ids.push(other_user_id);
                common_quest_ids = intersection;
            }
        }

        // 2人以上でマッチング成功
        if matched_user_ids.len() >= 2 {
            info!(
                guild_id,
                month,
                day,
                hour,
                user_count = matched_user_ids.len(),
                quest_count = common_quest_ids.len(),
                "マッチング成功"
            );

            // 既存のマッチング済み募集があるかチェック
            let existing = self
                .matched_repo
                .find_by_datetime(txn, guild_id, month, day, hour)
                .await?;

            return Ok(Some(MatchResult {
                month,
                day,
                hour,
                user_ids: matched_user_ids,
                common_quest_ids: common_quest_ids.into_iter().collect(),
                existing_matched_id: existing.map(|m| m.id),
            }));
        }

        debug!(
            guild_id,
            user_id, month, day, hour, "マッチング条件を満たしていません"
        );
        Ok(None)
    }

    /// 希望クエスト追加時にマッチングを実行
    ///
    /// 追加されたクエストを希望している他のユーザーを検索し、
    /// 参加時間が重なる場合をマッチングとして検出する
    pub async fn check_match_by_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
    ) -> Result<Vec<MatchResult>> {
        debug!(
            guild_id,
            user_id, quest_id, "希望クエスト追加によるマッチングをチェックします"
        );

        // 同じクエストを希望している他のユーザーを取得
        let quest_users = self
            .user_quest_repo
            .find_users_by_quest(txn, guild_id, quest_id)
            .await?;

        let other_user_ids: Vec<i64> = quest_users
            .iter()
            .filter(|q| q.user_id != user_id)
            .map(|q| q.user_id)
            .collect();

        if other_user_ids.is_empty() {
            debug!(
                guild_id,
                user_id, quest_id, "同じクエストを希望している他のユーザーがいません"
            );
            return Ok(vec![]);
        }

        // 自分の参加可能時間を取得
        let my_times = self
            .participant_repo
            .find_by_user(txn, guild_id, user_id)
            .await?;

        if my_times.is_empty() {
            debug!(guild_id, user_id, "参加可能時間が登録されていません");
            return Ok(vec![]);
        }

        // 日時ごとにマッチングをチェック
        let mut results = vec![];
        let mut checked_datetimes: HashSet<(i32, i32, i32)> = HashSet::new();

        for my_time in &my_times {
            let datetime_key = (my_time.month, my_time.day, my_time.hour);
            if checked_datetimes.contains(&datetime_key) {
                continue;
            }
            checked_datetimes.insert(datetime_key);

            // その日時に参加可能な他のユーザーを取得
            let participants = self
                .participant_repo
                .find_users_by_datetime(txn, guild_id, my_time.month, my_time.day, my_time.hour)
                .await?;

            // 他のユーザーで同じクエストを希望しているユーザーを探す
            let other_participants: Vec<i64> = participants
                .iter()
                .filter(|p| p.user_id != user_id && other_user_ids.contains(&p.user_id))
                .map(|p| p.user_id)
                .collect();

            if !other_participants.is_empty() {
                let mut matched_user_ids = vec![user_id];
                matched_user_ids.extend(other_participants);

                // 既存のマッチング済み募集があるかチェック
                let existing = self
                    .matched_repo
                    .find_by_datetime(txn, guild_id, my_time.month, my_time.day, my_time.hour)
                    .await?;

                info!(
                    guild_id,
                    month = my_time.month,
                    day = my_time.day,
                    hour = my_time.hour,
                    user_count = matched_user_ids.len(),
                    "マッチング成功"
                );

                results.push(MatchResult {
                    month: my_time.month,
                    day: my_time.day,
                    hour: my_time.hour,
                    user_ids: matched_user_ids,
                    common_quest_ids: vec![quest_id],
                    existing_matched_id: existing.map(|m| m.id),
                });
            }
        }

        Ok(results)
    }

    /// 指定された日時でマッチング可能なユーザーとクエストを検索
    ///
    /// 日時が固定された状態で、どのユーザー間でどのクエストでマッチングできるかを調べる
    pub async fn find_matching_candidates(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Option<MatchResult>> {
        debug!(guild_id, month, day, hour, "マッチング候補を検索します");

        // 指定日時に参加可能なユーザーを取得
        let participants = self
            .participant_repo
            .find_users_by_datetime(txn, guild_id, month, day, hour)
            .await?;

        if participants.len() < 2 {
            debug!(guild_id, month, day, hour, "参加者が2人未満です");
            return Ok(None);
        }

        let user_ids: Vec<i64> = participants.iter().map(|p| p.user_id).collect();

        // 各ユーザーの希望クエストを取得
        let mut user_quests: HashMap<i64, HashSet<i32>> = HashMap::new();
        for user_id in &user_ids {
            let quests = self
                .user_quest_repo
                .find_by_user(txn, guild_id, *user_id)
                .await?;
            let quest_ids: HashSet<i32> = quests.iter().map(|q| q.quest_id).collect();
            user_quests.insert(*user_id, quest_ids);
        }

        // 2人以上が共通して希望しているクエストを探す
        let mut quest_user_count: HashMap<i32, Vec<i64>> = HashMap::new();
        for (user_id, quest_ids) in &user_quests {
            for quest_id in quest_ids {
                quest_user_count
                    .entry(*quest_id)
                    .or_default()
                    .push(*user_id);
            }
        }

        // 2人以上が希望しているクエストを抽出
        let matching_quests: Vec<i32> = quest_user_count
            .iter()
            .filter(|(_, users)| users.len() >= 2)
            .map(|(quest_id, _)| *quest_id)
            .collect();

        if matching_quests.is_empty() {
            debug!(guild_id, month, day, hour, "共通のクエストがありません");
            return Ok(None);
        }

        // マッチングするユーザーを集める（少なくとも1つの共通クエストがある）
        let matched_user_ids: HashSet<i64> = matching_quests
            .iter()
            .flat_map(|quest_id| quest_user_count.get(quest_id).unwrap())
            .copied()
            .collect();

        // 既存のマッチング済み募集があるかチェック
        let existing = self
            .matched_repo
            .find_by_datetime(txn, guild_id, month, day, hour)
            .await?;

        Ok(Some(MatchResult {
            month,
            day,
            hour,
            user_ids: matched_user_ids.into_iter().collect(),
            common_quest_ids: matching_quests,
            existing_matched_id: existing.map(|m| m.id),
        }))
    }
}
