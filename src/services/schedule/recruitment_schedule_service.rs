use crate::models::entities::guild_master::{
    battle_recruitment_schedule_days, battle_recruitment_schedules,
};
use crate::types::Result;
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday};
use std::collections::HashSet;
use tracing::debug;

/// マルチ募集スケジュールサービス
pub struct RecruitmentScheduleService;

/// 計算された募集日時
#[derive(Debug, Clone)]
pub struct CalculatedRecruitmentTime {
    pub schedule_id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_at: DateTime<Utc>,
    pub recruit_start_at: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub note: Option<String>,
}

impl Default for RecruitmentScheduleService {
    fn default() -> Self {
        Self::new()
    }
}

impl RecruitmentScheduleService {
    pub fn new() -> Self {
        Self
    }

    /// 次回募集日時を計算
    /// from から to までの範囲内で、指定された曜日と時刻に該当する募集日時を計算
    ///
    /// # 注意
    /// DBに保存されている時刻と曜日は既にUTCに変換済みです
    pub fn calculate_next_recruitment_times(
        &self,
        schedule: &battle_recruitment_schedules::Model,
        days: &[battle_recruitment_schedule_days::Model],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CalculatedRecruitmentTime>> {
        debug!(
            schedule_id = schedule.id,
            from = %from,
            to = %to,
            "次回募集日時を計算します"
        );

        let mut result = Vec::new();

        // 曜日情報から対象曜日を抽出
        let target_weekdays = self.parse_day_of_weeks(days)?;

        debug!(
            schedule_id = schedule.id,
            weekdays = ?target_weekdays,
            "対象曜日を取得しました"
        );

        // quest_start_timeをNaiveTimeに変換
        let quest_start_time = schedule.quest_start_time;

        // recruit_start_timeを取得（NULLの場合はquest_start_timeを使用）
        let recruit_start_time = schedule.recruit_start_time.unwrap_or(quest_start_time);

        // fromからtoまでの各日について、対象曜日かどうかチェック
        // DBの値は既にUTC、そのまま使用
        let mut current_date = from.date_naive();
        let to_date = to.date_naive();

        while current_date <= to_date {
            let weekday = current_date.weekday();

            // 対象曜日かチェック（0=毎日 or 対応する曜日）
            if target_weekdays.contains(&0)
                || target_weekdays.contains(&Self::weekday_to_number(weekday))
            {
                // クエスト開始日時を計算（UTC時刻として直接使用）
                let quest_start_datetime = current_date
                    .and_time(Self::time_time_to_naive_time(quest_start_time))
                    .and_utc();

                // 募集開始日時を計算（recruit_start_day_offsetを考慮、UTC時刻として直接使用）
                let recruit_date =
                    current_date - Duration::days(schedule.recruit_start_day_offset as i64);
                let recruit_start_datetime = recruit_date
                    .and_time(Self::time_time_to_naive_time(recruit_start_time))
                    .and_utc();

                // 募集開始日時がfromとtoの範囲内かチェック
                if recruit_start_datetime >= from && recruit_start_datetime < to {
                    debug!(
                        schedule_id = schedule.id,
                        recruit_start_at = %recruit_start_datetime,
                        quest_start_at = %quest_start_datetime,
                        "募集日時を計算しました"
                    );

                    result.push(CalculatedRecruitmentTime {
                        schedule_id: schedule.id,
                        guild_id: schedule.guild_id,
                        channel_id: schedule.channel_id,
                        quest_id: schedule.quest_id,
                        battle_style_id: schedule.battle_style_id,
                        quest_start_at: quest_start_datetime,
                        recruit_start_at: recruit_start_datetime,
                        max_participants: schedule.max_participants,
                        note: schedule.note.clone(),
                    });
                }
            }

            // 次の日へ
            current_date += Duration::days(1);
        }

        debug!(
            schedule_id = schedule.id,
            count = result.len(),
            "募集日時の計算が完了しました"
        );

        Ok(result)
    }

    /// 入力値のバリデーション
    pub fn validate_schedule_input(
        &self,
        day_of_weeks: &[i32],
        quest_start_time: NaiveTime,
        recruit_start_day_offset: i32,
        recruit_start_time: Option<NaiveTime>,
    ) -> Result<()> {
        debug!("入力値のバリデーションを実行します");

        // 曜日の妥当性チェック（0-7の範囲）
        for &day in day_of_weeks {
            if !(0..=7).contains(&day) {
                return Err(crate::types::AppError::Business {
                    message: format!("無効な曜日です: {day}（0-7の範囲で指定してください）"),
                });
            }
        }

        // 曜日の重複チェック
        let unique_days: HashSet<i32> = day_of_weeks.iter().copied().collect();
        if unique_days.len() != day_of_weeks.len() {
            return Err(crate::types::AppError::Business {
                message: "曜日が重複しています".to_string(),
            });
        }

        // 「毎日」と他の曜日の組み合わせチェック
        if unique_days.contains(&0) && unique_days.len() > 1 {
            return Err(crate::types::AppError::Business {
                message: "「毎日」は他の曜日と組み合わせることができません".to_string(),
            });
        }

        // recruit_start_day_offsetの妥当性チェック（0-7の範囲）
        if !(0..=7).contains(&recruit_start_day_offset) {
            return Err(crate::types::AppError::Business {
                message: format!(
                    "無効な募集開始日オフセットです: {recruit_start_day_offset}（0-7の範囲で指定してください）"
                ),
            });
        }

        // 募集開始時刻とクエスト開始時刻の整合性チェック
        if let Some(recruit_time) = recruit_start_time
            && recruit_start_day_offset == 0
            && recruit_time >= quest_start_time
        {
            return Err(crate::types::AppError::Business {
                message: "当日募集の場合、募集開始時刻はクエスト開始時刻より前である必要があります"
                    .to_string(),
            });
        }

        debug!("入力値のバリデーションが完了しました");
        Ok(())
    }

    /// 曜日情報から対象曜日の数値リストを抽出
    fn parse_day_of_weeks(
        &self,
        days: &[battle_recruitment_schedule_days::Model],
    ) -> Result<Vec<i32>> {
        Ok(days.iter().map(|d| d.day_of_week).collect())
    }

    /// chrono::WeekdayをDB用の数値に変換
    /// 1=月、2=火、3=水、4=木、5=金、6=土、7=日
    fn weekday_to_number(weekday: Weekday) -> i32 {
        match weekday {
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
            Weekday::Sun => 7,
        }
    }

    /// TimeTime（SeaORM型）をNaiveTime（chrono型）に変換
    fn time_time_to_naive_time(time: sea_orm::prelude::TimeTime) -> NaiveTime {
        NaiveTime::from_hms_opt(
            time.hour() as u32,
            time.minute() as u32,
            time.second() as u32,
        )
        .unwrap_or(NaiveTime::MIN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_to_number() {
        assert_eq!(
            RecruitmentScheduleService::weekday_to_number(Weekday::Mon),
            1
        );
        assert_eq!(
            RecruitmentScheduleService::weekday_to_number(Weekday::Sun),
            7
        );
    }

    #[test]
    fn test_validate_schedule_input_valid() {
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1, 3, 5], // 月水金
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_schedule_input_duplicate_days() {
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1, 1, 3], // 重複あり
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schedule_input_everyday_with_others() {
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[0, 1], // 毎日と月曜日
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schedule_input_invalid_time_after() {
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1],
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            Some(NaiveTime::from_hms_opt(23, 0, 0).unwrap()), // 募集開始時刻がクエスト開始時刻より後
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schedule_input_invalid_time_equal() {
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1],
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()), // 募集開始時刻とクエスト開始時刻が同じ
        );
        assert!(result.is_err());
    }
}
