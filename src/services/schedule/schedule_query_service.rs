use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::{
    BattleRecruitmentScheduleRepository, NotificationRepository,
};
use crate::repository::quests_repository::QuestRepository;
use crate::services::schedule::convert_utc_days_and_time_to_local;
use crate::types::Result;
use chrono::{DateTime, Timelike, Utc};
use chrono_tz::Tz;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::collections::HashMap;

/// スケジュール一覧項目
#[derive(Debug, Clone)]
pub struct ScheduleListItem {
    pub id: i32,
    pub name: String,
    pub quest_name: String,
    pub days_str: String,
    pub timezone: Tz,
    pub quest_start_hour: u32,
    pub quest_start_minute: u32,
    pub recruit_day_offset: i32,
    pub recruit_time_str: String,
    pub created_by: i64,
    pub is_enabled: bool,
}

/// スケジュール統計（通知統計）
#[derive(Debug, Clone)]
pub struct ScheduleStats {
    pub total_count: usize,
    pub message_type_counts: HashMap<String, usize>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// スケジュールクエリサービス
///
/// スケジュール一覧取得・統計取得のビジネスロジックを担当するサービス。
pub struct ScheduleQueryService;

impl ScheduleQueryService {
    pub fn new() -> Self {
        Self
    }

    /// ユーザーが作成したスケジュールを取得（オートコンプリート用）
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `user_id`: ユーザーID
    ///
    /// # 戻り値
    /// スケジュールと曜日のタプルのベクタ
    pub async fn get_schedules_by_user(
        &self,
        txn: &DatabaseTransaction,
        user_id: i64,
    ) -> Result<
        Vec<(
            crate::models::entities::battle_recruitment_schedules::Model,
            Vec<crate::models::entities::battle_recruitment_schedule_days::Model>,
        )>,
    > {
        let schedule_repo = BattleRecruitmentScheduleRepository::new();
        schedule_repo.find_by_created_by(txn, user_id).await
    }

    /// スケジュール一覧を取得
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `db`: データベース接続（クエスト名取得用）
    /// - `guild_id`: ギルドID
    /// - `user_id`: ユーザーID（show_all=falseの場合のみ使用）
    /// - `show_all`: 全員のスケジュールを表示するか
    /// - `timezone`: タイムゾーン
    ///
    /// # 戻り値
    /// スケジュール一覧項目のベクタ
    pub async fn get_schedule_list(
        &self,
        txn: &DatabaseTransaction,
        db: &DatabaseConnection,
        guild_id: i64,
        user_id: i64,
        show_all: bool,
        timezone: Tz,
    ) -> Result<Vec<ScheduleListItem>> {
        let schedule_repo = BattleRecruitmentScheduleRepository::new();
        let quest_repo = SeaOrmQuestRepository::new();

        // スケジュールを取得
        let schedules = if show_all {
            schedule_repo.find_by_guild_id(txn, guild_id).await?
        } else {
            schedule_repo.find_by_created_by(txn, user_id).await?
        };

        // スケジュールを整形
        let mut items = Vec::new();
        for (schedule, days) in schedules {
            // クエスト名を取得
            let quest_name = match quest_repo.get_by_target_id(db, schedule.quest_id).await {
                Ok(Some(quest)) => quest.name,
                _ => format!("クエストID {}", schedule.quest_id),
            };

            // UTC曜日・時刻をローカル曜日・時刻に変換
            let utc_days: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
            let (local_days, local_quest_time) =
                convert_utc_days_and_time_to_local(&utc_days, schedule.quest_start_time, timezone)?;

            // 曜日を文字列に変換（ローカル曜日）
            let days_str = Self::format_days(&local_days);

            // 募集開始時刻を表示（ローカル時刻）
            let recruit_time_str = if let Some(recruit_time_utc) = schedule.recruit_start_time {
                let (_, local_recruit_time) =
                    convert_utc_days_and_time_to_local(&utc_days, recruit_time_utc, timezone)?;
                format!(
                    "{:02}:{:02}",
                    local_recruit_time.hour(),
                    local_recruit_time.minute()
                )
            } else {
                format!(
                    "{:02}:{:02}（クエスト開始時刻と同じ）",
                    local_quest_time.hour(),
                    local_quest_time.minute()
                )
            };

            items.push(ScheduleListItem {
                id: schedule.id,
                name: schedule.name,
                quest_name,
                days_str,
                timezone,
                quest_start_hour: local_quest_time.hour(),
                quest_start_minute: local_quest_time.minute(),
                recruit_day_offset: schedule.recruit_start_day_offset,
                recruit_time_str,
                created_by: schedule.created_by,
                is_enabled: schedule.is_enabled,
            });
        }

        Ok(items)
    }

    /// 通知統計を取得
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `guild_id`: ギルドID
    /// - `from`: 開始日時
    /// - `to`: 終了日時
    ///
    /// # 戻り値
    /// スケジュール統計
    pub async fn get_notification_stats(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ScheduleStats> {
        let notification_repo = NotificationRepository::new();

        // 統計を取得
        let all_notifications = notification_repo
            .find_by_datetime_range_with_txn(txn, from, to)
            .await?;

        // ギルドでフィルタ
        let notifications: Vec<_> = all_notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect();

        let total_count = notifications.len();

        // メッセージタイプ別の集計
        let mut message_type_counts = HashMap::new();
        for notification in &notifications {
            *message_type_counts
                .entry(notification.message_text_id.clone())
                .or_insert(0) += 1;
        }

        Ok(ScheduleStats {
            total_count,
            message_type_counts,
            from,
            to,
        })
    }

    /// 曜日を文字列に変換
    fn format_days(days: &[i32]) -> String {
        let day_names: Vec<String> = days
            .iter()
            .map(|&d| match d {
                0 => "毎日".to_string(),
                1 => "月".to_string(),
                2 => "火".to_string(),
                3 => "水".to_string(),
                4 => "木".to_string(),
                5 => "金".to_string(),
                6 => "土".to_string(),
                7 => "日".to_string(),
                _ => format!("不明({d})"),
            })
            .collect();

        day_names.join(", ")
    }
}

impl Default for ScheduleQueryService {
    fn default() -> Self {
        Self::new()
    }
}
