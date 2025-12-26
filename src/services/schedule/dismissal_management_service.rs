use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::database::schedule::{
    BattleRecruitmentDismissalRepository, ScheduledTaskDismissalRepository, ScheduledTaskRepository,
};
use crate::services::recruitment::dismissal_time_parser_service::ParsedDismissalTime;
use crate::types::Result;
use chrono::{DateTime, Duration, Utc};
use sea_orm::DatabaseTransaction;
use tracing::{debug, info};

/// 解散日時管理サービス
/// 解散日時の登録・削除を管理する
pub struct DismissalManagementService;

impl Default for DismissalManagementService {
    fn default() -> Self {
        Self::new()
    }
}

impl DismissalManagementService {
    pub fn new() -> Self {
        Self
    }

    /// マルチ募集の解散時刻を登録
    ///
    /// # 引数
    /// - `txn`: トランザクション
    /// - `recruitment_id`: 募集ID
    /// - `dismissal_times`: パース済み解散時刻のリスト
    /// - `departure_time`: 出発日時
    /// - `guild_id`: ギルドID
    /// - `channel_id`: チャンネルID
    pub async fn create_recruitment_dismissals(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        dismissal_times: Vec<ParsedDismissalTime>,
        departure_time: DateTime<Utc>,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<()> {
        debug!(
            recruitment_id,
            dismissal_count = dismissal_times.len(),
            "マルチ募集の解散時刻を登録します"
        );

        let dismissal_repo = BattleRecruitmentDismissalRepository::new();
        let scheduled_task_repo = ScheduledTaskRepository::new();
        let task_dismissal_repo = ScheduledTaskDismissalRepository::new();

        for dismissal_time in &dismissal_times {
            // 1. 解散日時を計算
            let dismissal_datetime = match &dismissal_time {
                ParsedDismissalTime::Absolute { datetime, .. } => *datetime,
                ParsedDismissalTime::Relative {
                    days,
                    hours,
                    minutes,
                    ..
                } => {
                    let duration = Duration::days(*days as i64)
                        + Duration::hours(*hours as i64)
                        + Duration::minutes(*minutes as i64);
                    departure_time - duration
                }
            };

            // 2. battle_recruitment_dismissals テーブルにレコード作成
            let dismissal_record = match dismissal_time {
                ParsedDismissalTime::Absolute {
                    input_value,
                    datetime,
                } => {
                    dismissal_repo
                        .create_absolute(txn, recruitment_id, input_value.clone(), *datetime)
                        .await?
                }
                ParsedDismissalTime::Relative {
                    input_value,
                    days,
                    hours,
                    minutes,
                } => {
                    dismissal_repo
                        .create_relative(
                            txn,
                            recruitment_id,
                            input_value.clone(),
                            *days,
                            *hours,
                            *minutes,
                        )
                        .await?
                }
            };

            // 3. scheduled_tasks テーブルにタスク作成（task_type=2: Dismissal）
            let scheduled_task = scheduled_task_repo
                .create(
                    txn,
                    dismissal_datetime,
                    ScheduledTaskType::Dismissal.as_i32(),
                    Some(guild_id),
                    Some(channel_id),
                )
                .await?;

            // 4. scheduled_task_dismissals テーブルに紐付け作成
            task_dismissal_repo
                .create(txn, scheduled_task.id, dismissal_record.id)
                .await?;

            info!(
                recruitment_id,
                dismissal_id = dismissal_record.id,
                task_id = scheduled_task.id,
                dismissal_datetime = %dismissal_datetime,
                "解散時刻を登録しました"
            );
        }

        info!(
            recruitment_id,
            count = dismissal_times.len(),
            "マルチ募集の解散時刻登録が完了しました"
        );

        Ok(())
    }

    /// マルチ募集の解散時刻を削除
    ///
    /// # 引数
    /// - `txn`: トランザクション
    /// - `recruitment_id`: 募集ID
    ///
    /// # 戻り値
    /// 削除した解散時刻の数
    pub async fn delete_recruitment_dismissals(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<usize> {
        use tracing::debug;

        let dismissal_repo = BattleRecruitmentDismissalRepository::new();
        let scheduled_task_repo = ScheduledTaskRepository::new();
        let task_dismissal_repo = ScheduledTaskDismissalRepository::new();

        debug!(
            recruitment_id,
            "マルチ募集の解散時刻とリレーションを削除します"
        );

        // 募集に紐づく解散時刻を検索
        let dismissals = dismissal_repo
            .find_by_recruitment_id(txn, recruitment_id)
            .await?;

        let dismissals_count = dismissals.len();

        debug!(
            recruitment_id,
            dismissals_count, "募集に紐づく解散時刻とリレーションを削除します"
        );

        // 外部キー制約を考慮し、scheduled_task_dismissals → scheduled_task → dismissal の順で削除
        for dismissal in dismissals {
            // scheduled_task_dismissals を検索して削除
            if let Some(task_dismissal_rel) = task_dismissal_repo
                .find_by_recruitment_dismissal_id(txn, dismissal.id)
                .await?
            {
                // scheduled_task を削除（CASCADE で scheduled_task_dismissals も削除される）
                scheduled_task_repo
                    .delete_by_id(txn, task_dismissal_rel.task_id)
                    .await?;
                debug!(
                    task_id = task_dismissal_rel.task_id,
                    "scheduled_taskを削除しました"
                );
            }

            // 解散時刻レコードは CASCADE で削除されるため、明示的な削除は不要
            debug!(
                dismissal_id = dismissal.id,
                "解散時刻レコードが削除されました（CASCADE）"
            );
        }

        info!(
            recruitment_id,
            deleted_count = dismissals_count,
            "募集に紐づく解散時刻とリレーションの削除が完了しました"
        );

        Ok(dismissals_count)
    }
}
