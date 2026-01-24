//! 参加状況確認Facade
//!
//! ユーザーの自動募集参加状況を取得する

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::{
    AutoRecruitmentParticipantRepository, AutoRecruitmentRepository, UserDesiredQuestRepository,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentParticipantRepository, SeaOrmAutoRecruitmentRepository,
    SeaOrmUserDesiredQuestRepository,
};
use crate::types::{AppError, AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 参加状況情報
#[derive(Debug)]
pub struct ParticipationStatus {
    /// 選択中のクエストID一覧
    pub quest_ids: Vec<i32>,
    /// 日時ごとの参加登録
    pub time_slots: Vec<TimeSlot>,
}

/// 時間帯情報
#[derive(Debug)]
pub struct TimeSlot {
    /// 月
    pub month: i32,
    /// 日
    pub day: i32,
    /// 時間一覧
    pub hours: Vec<i32>,
}

/// 参加状況を取得
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
#[instrument(level = "info", skip(app_state))]
pub async fn get_participation_status(
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
) -> Result<ParticipationStatus> {
    info!(guild_id, user_id, "参加状況を取得します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let quest_repo = SeaOrmUserDesiredQuestRepository::new();
        let participant_repo = SeaOrmAutoRecruitmentParticipantRepository::new();

        // 自動募集設定を確認
        let _auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        // ユーザーのクエスト選択を取得
        let user_quests = quest_repo
            .find_by_user(&txn, guild_id as i64, user_id as i64)
            .await?;

        let quest_ids: Vec<i32> = user_quests.iter().map(|q| q.quest_id).collect();

        // ユーザーの時間選択を取得
        let user_participants = participant_repo
            .find_by_user(&txn, guild_id as i64, user_id as i64)
            .await?;

        // 日時ごとにグループ化
        let mut time_slot_map: std::collections::HashMap<(i32, i32), Vec<i32>> =
            std::collections::HashMap::new();

        for participant in user_participants {
            time_slot_map
                .entry((participant.month, participant.day))
                .or_default()
                .push(participant.hour);
        }

        // 日付順でソート
        let mut time_slots: Vec<TimeSlot> = time_slot_map
            .into_iter()
            .map(|((month, day), mut hours)| {
                hours.sort();
                TimeSlot { month, day, hours }
            })
            .collect();

        time_slots.sort_by_key(|slot| (slot.month, slot.day));

        Ok(ParticipationStatus {
            quest_ids,
            time_slots,
        })
    }
    .await;

    match result {
        Ok(status) => {
            txn.commit().await?;
            info!(
                guild_id,
                user_id,
                quest_count = status.quest_ids.len(),
                time_slot_count = status.time_slots.len(),
                "参加状況を取得しました"
            );
            Ok(status)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, "参加状況の取得に失敗しました");
            Err(e)
        }
    }
}
