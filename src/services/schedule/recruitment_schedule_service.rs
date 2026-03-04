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

    /// 指定した募集開始日時に一致する実行回の募集日時情報を解決
    ///
    /// `task.schedule_datetime` のような既存タスク時刻から、
    /// 当該実行回の `CalculatedRecruitmentTime` を復元するために使用する。
    pub fn resolve_recruitment_time_by_recruit_start_at(
        &self,
        schedule: &battle_recruitment_schedules::Model,
        days: &[battle_recruitment_schedule_days::Model],
        recruit_start_at: DateTime<Utc>,
    ) -> Result<Option<CalculatedRecruitmentTime>> {
        // recruit_start_day_offset が最大7日であるため、前後8日を探索範囲にする
        let search_from = recruit_start_at - Duration::days(8);
        let search_to = recruit_start_at + Duration::days(8);

        debug!(
            schedule_id = schedule.id,
            recruit_start_at = %recruit_start_at,
            search_from = %search_from,
            search_to = %search_to,
            "募集開始日時から実行回を解決します"
        );

        let next_times =
            self.calculate_next_recruitment_times(schedule, days, search_from, search_to)?;

        Ok(next_times
            .into_iter()
            .find(|time| time.recruit_start_at == recruit_start_at))
    }

    /// 現在時刻時点で「募集開始済みかつ出発前」の実行回を解決
    ///
    /// Bot停止復旧時など、過去タスクをスキップした後に
    /// 現在募集可能な回が存在する場合の即時募集判定に使用する。
    pub fn resolve_executable_recruitment_time_at_now(
        &self,
        schedule: &battle_recruitment_schedules::Model,
        days: &[battle_recruitment_schedule_days::Model],
        now: DateTime<Utc>,
    ) -> Result<Option<CalculatedRecruitmentTime>> {
        // recruit_start_day_offset が最大7日のため、前後8日で十分に探索できる
        let search_from = now - Duration::days(8);
        let search_to = now + Duration::days(8);

        debug!(
            schedule_id = schedule.id,
            now = %now,
            search_from = %search_from,
            search_to = %search_to,
            "現在時刻で実行可能な募集日時を解決します"
        );

        let next_times =
            self.calculate_next_recruitment_times(schedule, days, search_from, search_to)?;

        Ok(next_times
            .into_iter()
            .filter(|time| time.recruit_start_at <= now && now < time.quest_start_at)
            .max_by_key(|time| time.recruit_start_at))
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
    use chrono::TimeZone;
    use sea_orm::prelude::TimeTime;

    fn test_schedule(
        recruit_start_day_offset: i32,
        quest_start_time: TimeTime,
        recruit_start_time: Option<TimeTime>,
    ) -> battle_recruitment_schedules::Model {
        let now = Utc::now();
        battle_recruitment_schedules::Model {
            id: 1,
            name: "test_schedule".to_string(),
            guild_id: 100,
            channel_id: 200,
            quest_id: 300,
            battle_style_id: 400,
            quest_start_time,
            recruit_start_day_offset,
            recruit_start_time,
            max_participants: Some(6),
            note: Some("test".to_string()),
            is_enabled: true,
            created_by: 999,
            created_at: now,
            updated_at: now,
        }
    }

    fn test_day(day_of_week: i32) -> battle_recruitment_schedule_days::Model {
        let now = Utc::now();
        battle_recruitment_schedule_days::Model {
            id: 1,
            schedule_id: 1,
            day_of_week,
            created_at: now,
            updated_at: now,
        }
    }

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

    #[test]
    fn test_resolve_recruitment_time_by_recruit_start_at_found() {
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            1,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)]; // 毎日

        let recruit_start_at = Utc.with_ymd_and_hms(2026, 3, 2, 20, 0, 0).single().unwrap();
        let resolved = service
            .resolve_recruitment_time_by_recruit_start_at(&schedule, &days, recruit_start_at)
            .unwrap()
            .unwrap();

        let expected_quest_start_at = Utc.with_ymd_and_hms(2026, 3, 3, 21, 0, 0).single().unwrap();
        assert_eq!(resolved.recruit_start_at, recruit_start_at);
        assert_eq!(resolved.quest_start_at, expected_quest_start_at);
    }

    #[test]
    fn test_resolve_recruitment_time_by_recruit_start_at_not_found() {
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)]; // 毎日

        let unmatched_recruit_start_at = Utc
            .with_ymd_and_hms(2026, 3, 2, 20, 30, 0)
            .single()
            .unwrap();
        let resolved = service
            .resolve_recruitment_time_by_recruit_start_at(
                &schedule,
                &days,
                unmatched_recruit_start_at,
            )
            .unwrap();

        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_executable_recruitment_time_at_now_between_recruit_and_departure() {
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)];
        let now = Utc
            .with_ymd_and_hms(2026, 3, 2, 20, 30, 0)
            .single()
            .unwrap();

        let resolved = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, now)
            .unwrap()
            .unwrap();

        assert_eq!(
            resolved.recruit_start_at,
            Utc.with_ymd_and_hms(2026, 3, 2, 20, 0, 0).single().unwrap()
        );
        assert_eq!(
            resolved.quest_start_at,
            Utc.with_ymd_and_hms(2026, 3, 2, 21, 0, 0).single().unwrap()
        );
    }

    #[test]
    fn test_resolve_executable_recruitment_time_at_now_before_recruit_start_returns_none() {
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)];
        let now = Utc.with_ymd_and_hms(2026, 3, 2, 19, 0, 0).single().unwrap();

        let resolved = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, now)
            .unwrap();

        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_executable_recruitment_time_at_now_after_departure_returns_none() {
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)];
        let now = Utc.with_ymd_and_hms(2026, 3, 2, 21, 1, 0).single().unwrap();

        let resolved = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, now)
            .unwrap();

        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_executable_recruitment_time_at_now_pattern_b_c_window() {
        // A: 03/01 20:00-21:00, B: 03/02 20:00-21:00, C: 03/03 20:00-21:00 を想定
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)];

        // B出発後〜C募集開始前は即時実行対象なし
        let before_c_recruit = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).single().unwrap();
        let resolved_before_c = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, before_c_recruit)
            .unwrap();
        assert!(resolved_before_c.is_none());

        // C募集開始後〜C出発前はCが即時実行対象
        let during_c = Utc
            .with_ymd_and_hms(2026, 3, 3, 20, 30, 0)
            .single()
            .unwrap();
        let resolved_during_c = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, during_c)
            .unwrap()
            .unwrap();
        assert_eq!(
            resolved_during_c.recruit_start_at,
            Utc.with_ymd_and_hms(2026, 3, 3, 20, 0, 0).single().unwrap()
        );
    }

    #[test]
    fn test_resolve_executable_recruitment_time_at_now_startup_timing_patterns() {
        // 日次スケジュール（毎日 20:00募集開始 / 21:00出発）
        // A: 03/01, B: 03/02, C: 03/03
        let service = RecruitmentScheduleService::new();
        let schedule = test_schedule(
            0,
            TimeTime::from_hms(21, 0, 0).unwrap(),
            Some(TimeTime::from_hms(20, 0, 0).unwrap()),
        );
        let days = vec![test_day(0)];

        // 1) A出発時間〜B募集開始時間: 即時実行対象なし
        let pattern_1_now = Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).single().unwrap();
        let p1 = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, pattern_1_now)
            .unwrap();
        assert!(p1.is_none());

        // 2) B募集開始時間〜B出発時間: Bが即時実行対象
        let pattern_2_now = Utc
            .with_ymd_and_hms(2026, 3, 2, 20, 30, 0)
            .single()
            .unwrap();
        let p2 = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, pattern_2_now)
            .unwrap()
            .unwrap();
        assert_eq!(
            p2.recruit_start_at,
            Utc.with_ymd_and_hms(2026, 3, 2, 20, 0, 0).single().unwrap()
        );

        // 3) B出発時間〜C募集開始時間: 即時実行対象なし
        let pattern_3_now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).single().unwrap();
        let p3 = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, pattern_3_now)
            .unwrap();
        assert!(p3.is_none());

        // 4) C募集開始時間〜C出発時間: Cが即時実行対象
        let pattern_4_now = Utc
            .with_ymd_and_hms(2026, 3, 3, 20, 30, 0)
            .single()
            .unwrap();
        let p4 = service
            .resolve_executable_recruitment_time_at_now(&schedule, &days, pattern_4_now)
            .unwrap()
            .unwrap();
        assert_eq!(
            p4.recruit_start_at,
            Utc.with_ymd_and_hms(2026, 3, 3, 20, 0, 0).single().unwrap()
        );
    }
}
