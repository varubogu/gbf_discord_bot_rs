use crate::repository::auto_recruitment::{
    AutoRecruitmentParticipantRepository, AutoRecruitmentRepository, UserDesiredQuestRepository,
};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;

/// ユーザーの参加状況データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipationStatusData {
    pub quest_ids: Vec<i32>,
    pub time_slots: Vec<TimeSlotData>,
}

/// 参加時間帯データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSlotData {
    pub month: i32,
    pub day: i32,
    pub hours: Vec<i32>,
}

/// 参加状況確認サービス
///
/// 自動募集設定の存在確認と、参加状況の集約を担当する。
pub struct ParticipationStatusService<AR, UQ, PR>
where
    AR: AutoRecruitmentRepository,
    UQ: UserDesiredQuestRepository,
    PR: AutoRecruitmentParticipantRepository,
{
    auto_recruitment_repo: AR,
    user_desired_quest_repo: UQ,
    participant_repo: PR,
}

impl<AR, UQ, PR> ParticipationStatusService<AR, UQ, PR>
where
    AR: AutoRecruitmentRepository,
    UQ: UserDesiredQuestRepository,
    PR: AutoRecruitmentParticipantRepository,
{
    pub fn new(
        auto_recruitment_repo: AR,
        user_desired_quest_repo: UQ,
        participant_repo: PR,
    ) -> Self {
        Self {
            auto_recruitment_repo,
            user_desired_quest_repo,
            participant_repo,
        }
    }

    /// ユーザーの参加状況を取得
    pub async fn get_participation_status(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<ParticipationStatusData> {
        self.auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        let user_quests = self
            .user_desired_quest_repo
            .find_by_user(txn, guild_id, user_id)
            .await?;
        let quest_ids: Vec<i32> = user_quests.iter().map(|q| q.quest_id).collect();

        let user_participants = self
            .participant_repo
            .find_by_user(txn, guild_id, user_id)
            .await?;

        let mut time_slot_map: HashMap<(i32, i32), Vec<i32>> = HashMap::new();
        for participant in user_participants {
            time_slot_map
                .entry((participant.month, participant.day))
                .or_default()
                .push(participant.hour);
        }

        let mut time_slots: Vec<TimeSlotData> = time_slot_map
            .into_iter()
            .map(|((month, day), mut hours)| {
                hours.sort();
                TimeSlotData { month, day, hours }
            })
            .collect();
        time_slots.sort_by_key(|slot| (slot.month, slot.day));

        Ok(ParticipationStatusData {
            quest_ids,
            time_slots,
        })
    }
}
