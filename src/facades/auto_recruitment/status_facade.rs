//! 参加状況確認Facade
//!
//! ユーザーの自動募集参加状況を取得する

use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::auto_recruitment::ParticipationStatusService;
use crate::types::{AppState, Result};
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
        let service = ParticipationStatusService::new(
            app_state.repositories.auto_recruitment,
            app_state.repositories.user_desired_quest,
            app_state.repositories.auto_recruitment_participant,
        );
        let status = service
            .get_participation_status(&txn, guild_id as i64, user_id as i64)
            .await?;

        Ok(ParticipationStatus {
            quest_ids: status.quest_ids,
            time_slots: status
                .time_slots
                .into_iter()
                .map(|slot| TimeSlot {
                    month: slot.month,
                    day: slot.day,
                    hours: slot.hours,
                })
                .collect(),
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
