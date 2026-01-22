//! 時間選択Facade
//!
//! ユーザーの参加可能時間選択を処理する

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::{
    AutoRecruitmentParticipantRepository, AutoRecruitmentRepository, UserDesiredQuestRepository,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentParticipantRepository, SeaOrmAutoRecruitmentRepository,
    SeaOrmUserDesiredQuestRepository,
};
use crate::types::{AppError, AppState, Result};
use poise::serenity_prelude::Context;
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 時間選択結果
#[derive(Debug)]
pub enum TimeSelectionResult {
    /// 登録完了
    Registered {
        month: i32,
        day: i32,
        hours: Vec<i32>,
    },
}

/// 時間選択を処理
///
/// # 引数
/// * `_ctx` - Discord Context（将来の拡張用）
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
/// * `month` - 月
/// * `day` - 日
/// * `hours` - 選択された時間のリスト
#[instrument(level = "info", skip(_ctx, app_state))]
pub async fn handle_time_selection(
    _ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    month: i32,
    day: i32,
    hours: Vec<i32>,
) -> Result<TimeSelectionResult> {
    info!(
        guild_id,
        user_id,
        month,
        day,
        hour_count = hours.len(),
        "時間選択を処理します"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let participant_repo = SeaOrmAutoRecruitmentParticipantRepository::new();
        let quest_repo = SeaOrmUserDesiredQuestRepository::new();

        // 自動募集設定を確認
        let _auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        // ユーザーがクエストを登録しているか確認
        let user_quests = quest_repo
            .find_by_user(&txn, guild_id as i64, user_id as i64)
            .await?;

        if user_quests.is_empty() {
            return Err(AppError::Business {
                message: "先にクエスト選択チャンネルで希望クエストを選択してください".to_string(),
            });
        }

        // 既存の時間登録を削除して新しく登録
        participant_repo
            .delete_all_by_user_date(&txn, guild_id as i64, user_id as i64, month, day)
            .await?;

        for hour in &hours {
            participant_repo
                .create(&txn, guild_id as i64, user_id as i64, month, day, *hour)
                .await?;
        }

        info!(
            guild_id,
            user_id,
            month,
            day,
            hours = ?hours,
            "時間選択を登録しました"
        );

        // TODO: マッチングチェックと通知は別サービスで実行

        Ok(TimeSelectionResult::Registered {
            month,
            day,
            hours: hours.clone(),
        })
    }
    .await;

    match result {
        Ok(res) => {
            txn.commit().await?;
            info!(guild_id, user_id, "時間選択処理が完了しました");
            Ok(res)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, "時間選択処理に失敗しました");
            Err(e)
        }
    }
}
