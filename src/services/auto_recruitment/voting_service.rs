//! 自動募集投票サービス
//!
//! マッチング成立後のクエスト投票を管理するサービス

use crate::models::entities::worker::matched_recruitment_votes;
use crate::repository::auto_recruitment::{
    MatchedRecruitmentChannelRepository, MatchedRecruitmentVoteRepository,
};
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// 投票結果
#[derive(Debug, Clone)]
pub enum VoteResult {
    /// クエストが決定した
    Decided { quest_id: i32 },
    /// 同数投票で再投票が必要
    Tie { quest_ids: Vec<i32> },
    /// まだ全員が投票していない
    Pending {
        voted_count: usize,
        total_count: usize,
    },
}

/// 自動募集投票サービス
pub struct VotingService<V, M>
where
    V: MatchedRecruitmentVoteRepository,
    M: MatchedRecruitmentChannelRepository,
{
    vote_repo: Arc<V>,
    matched_repo: Arc<M>,
}

impl<V, M> VotingService<V, M>
where
    V: MatchedRecruitmentVoteRepository,
    M: MatchedRecruitmentChannelRepository,
{
    pub fn new(vote_repo: Arc<V>, matched_repo: Arc<M>) -> Self {
        Self {
            vote_repo,
            matched_repo,
        }
    }

    /// 投票を登録または更新
    pub async fn vote(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        user_id: i64,
        quest_id: Option<i32>,
    ) -> Result<matched_recruitment_votes::Model> {
        debug!(matched_channel_id, user_id, ?quest_id, "投票を登録します");

        let vote = self
            .vote_repo
            .upsert(txn, matched_channel_id, user_id, quest_id)
            .await?;

        info!(matched_channel_id, user_id, ?quest_id, "投票を登録しました");
        Ok(vote)
    }

    /// 投票結果を集計してクエストを決定
    ///
    /// - 全員が「何でも良い」（quest_id = None）の場合はランダム決定
    /// - 「何でも良い」以外の投票で最多のクエストが1つなら決定
    /// - 同数の場合は再投票が必要
    pub async fn determine_quest(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        participant_user_ids: &[i64],
        candidate_quest_ids: &[i32],
    ) -> Result<VoteResult> {
        debug!(matched_channel_id, "投票結果を集計します");

        let votes = self
            .vote_repo
            .find_by_matched_channel_id(txn, matched_channel_id)
            .await?;

        let voted_count = votes.len();
        let total_count = participant_user_ids.len();

        // まだ全員が投票していない
        if voted_count < total_count {
            debug!(
                matched_channel_id,
                voted_count, total_count, "まだ全員が投票していません"
            );
            return Ok(VoteResult::Pending {
                voted_count,
                total_count,
            });
        }

        // 投票を集計
        let mut quest_votes: HashMap<i32, usize> = HashMap::new();
        let mut any_count = 0;

        for vote in &votes {
            if let Some(quest_id) = vote.quest_id {
                *quest_votes.entry(quest_id).or_insert(0) += 1;
            } else {
                any_count += 1;
            }
        }

        // 全員が「何でも良い」の場合
        if any_count == total_count {
            debug!(
                matched_channel_id,
                "全員が「何でも良い」を選択したため、ランダム決定します"
            );
            let quest_id = self.random_select(candidate_quest_ids);
            return Ok(VoteResult::Decided { quest_id });
        }

        // 「何でも良い」以外の投票で最多を決定
        if quest_votes.is_empty() {
            // 投票があるのにquest_votesが空ということはありえないが念のため
            let quest_id = self.random_select(candidate_quest_ids);
            return Ok(VoteResult::Decided { quest_id });
        }

        let max_votes = *quest_votes.values().max().unwrap();
        let top_quests: Vec<i32> = quest_votes
            .iter()
            .filter(|(_, count)| **count == max_votes)
            .map(|(quest_id, _)| *quest_id)
            .collect();

        if top_quests.len() == 1 {
            // 最多が1つなら決定
            let quest_id = top_quests[0];
            info!(matched_channel_id, quest_id, "クエストが決定しました");
            return Ok(VoteResult::Decided { quest_id });
        }

        // 同数の場合は再投票が必要
        info!(
            matched_channel_id,
            ?top_quests,
            "同数投票のため再投票が必要です"
        );
        Ok(VoteResult::Tie {
            quest_ids: top_quests,
        })
    }

    /// クエストをランダムに選択
    fn random_select(&self, quest_ids: &[i32]) -> i32 {
        if quest_ids.is_empty() {
            return 0;
        }
        if quest_ids.len() == 1 {
            return quest_ids[0];
        }
        // 簡易的なランダム選択（システム時刻を使用）
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as usize)
            .unwrap_or(0);
        let index = seed % quest_ids.len();
        quest_ids[index]
    }

    /// マッチング済み募集のクエストを決定して更新
    pub async fn finalize_quest(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
        quest_id: i32,
    ) -> Result<()> {
        debug!(matched_channel_id, quest_id, "クエストを確定します");

        self.matched_repo
            .decide_quest(txn, matched_channel_id, quest_id)
            .await?;

        info!(matched_channel_id, quest_id, "クエストを確定しました");
        Ok(())
    }

    /// 投票状況を取得
    pub async fn get_vote_status(
        &self,
        txn: &DatabaseTransaction,
        matched_channel_id: i32,
    ) -> Result<Vec<matched_recruitment_votes::Model>> {
        self.vote_repo
            .find_by_matched_channel_id(txn, matched_channel_id)
            .await
    }
}
