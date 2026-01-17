use crate::models::entities::master::{event_schedule_details, event_schedules};
use crate::types::Result;
use chrono::{DateTime, Duration, FixedOffset, NaiveTime, TimeZone, Utc};
use std::collections::HashMap;
use tracing::{debug, error, warn};

/// スケジュール計算サービス
pub struct ScheduleCalculator;

/// スケジュール計算結果
#[derive(Debug, Clone)]
pub struct CalculatedSchedule {
    pub schedule_datetime: DateTime<Utc>,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_text_id: String,
    pub event_schedule_id: uuid::Uuid,
    pub event_schedule_detail_id: uuid::Uuid,
}

impl ScheduleCalculator {
    pub fn new() -> Self {
        Self
    }

    /// イベントスケジュールと詳細から具体的なスケジュールを計算
    pub fn calculate_schedules(
        &self,
        event_schedules: Vec<event_schedules::Model>,
        event_schedule_details: Vec<event_schedule_details::Model>,
        guild_channels_by_type: HashMap<i32, Vec<(i64, i64)>>, // channel_type -> Vec<(guild_id, channel_id)>
    ) -> Result<Vec<CalculatedSchedule>> {
        debug!(
            event_count = event_schedules.len(),
            detail_count = event_schedule_details.len(),
            channel_types = guild_channels_by_type.len(),
            "スケジュール計算を開始します"
        );

        let mut results = Vec::new();

        for event_schedule in &event_schedules {
            // プロファイルに一致する詳細スケジュールを取得
            let matching_details: Vec<_> = event_schedule_details
                .iter()
                .filter(|detail| detail.profile == event_schedule.profile)
                .collect();

            debug!(
                profile = %event_schedule.profile,
                detail_count = matching_details.len(),
                "プロファイルに一致する詳細スケジュールを発見しました"
            );

            for detail in matching_details {
                // notification_channel_typeに対応するギルド・チャンネルを取得
                let guild_channels =
                    match guild_channels_by_type.get(&detail.notification_channel_type) {
                        Some(channels) => channels,
                        None => {
                            warn!(
                                channel_type = detail.notification_channel_type,
                                profile = %event_schedule.profile,
                                schedule_name = %detail.schedule_name,
                                "該当するchannel_typeのギルド・チャンネルが登録されていません"
                            );
                            continue;
                        }
                    };

                // 各ギルド・チャンネルに対してスケジュールを生成
                for (guild_id, channel_id) in guild_channels {
                    match self.calculate_datetimes(event_schedule, detail) {
                        Ok(schedule_datetimes) => {
                            for schedule_datetime in schedule_datetimes {
                                results.push(CalculatedSchedule {
                                    schedule_datetime,
                                    guild_id: *guild_id,
                                    channel_id: *channel_id,
                                    message_text_id: detail.message_text_id.clone(),
                                    event_schedule_id: event_schedule.id,
                                    event_schedule_detail_id: detail.id,
                                });
                            }
                        }
                        Err(e) => {
                            warn!(
                                error = %e,
                                profile = %event_schedule.profile,
                                schedule_name = %detail.schedule_name,
                                "スケジュール日時の計算に失敗しました"
                            );
                        }
                    }
                }
            }
        }

        debug!(
            result_count = results.len(),
            "スケジュール計算が完了しました"
        );
        Ok(results)
    }

    /// イベントスケジュールと詳細から具体的な日時を計算（複数日対応）
    fn calculate_datetimes(
        &self,
        event_schedule: &event_schedules::Model,
        detail: &event_schedule_details::Model,
    ) -> Result<Vec<DateTime<Utc>>> {
        // start_day_relativeをパースして日付オフセットのリストを取得
        let day_offsets = self.parse_start_day_relative(
            &detail.start_day_relative,
            event_schedule.start_at,
            event_schedule.end_at,
        )?;

        // timeをパース（例: "05:00:00", "23:59:59"）
        let time = self.parse_time(&detail.time)?;

        let mut results = Vec::new();
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();

        // イベント期間（JSTのNaiveDateTimeをUTCに変換）
        let start_at_utc = jst
            .from_local_datetime(&event_schedule.start_at)
            .single()
            .ok_or_else(|| crate::types::AppError::Validation {
                field: format!("イベント開始日時変換: {}", event_schedule.start_at),
            })?
            .with_timezone(&Utc);

        let end_at_utc = jst
            .from_local_datetime(&event_schedule.end_at)
            .single()
            .ok_or_else(|| crate::types::AppError::Validation {
                field: format!("イベント終了日時変換: {}", event_schedule.end_at),
            })?
            .with_timezone(&Utc);

        for day_offset in day_offsets {
            // イベント開始日時に日数オフセットを追加（JSTのまま計算）
            let target_datetime_jst = event_schedule.start_at + Duration::days(day_offset);

            // 日付と時刻を組み合わせる（JSTとして）
            let naive_datetime = target_datetime_jst.date().and_time(time);

            // JSTをUTCに変換（JST = UTC+9）
            let schedule_datetime_jst = jst
                .from_local_datetime(&naive_datetime)
                .single()
                .ok_or_else(|| crate::types::AppError::Validation {
                    field: format!("日時計算: {naive_datetime}"),
                })?;

            let schedule_datetime_utc = schedule_datetime_jst.with_timezone(&Utc);

            // 開始日前や終了日後の通知も許可するため、期間チェックは行わない
            results.push(schedule_datetime_utc);

            debug!(
                event_start_jst = %event_schedule.start_at,
                day_offset = day_offset,
                time = %detail.time,
                result_jst = %schedule_datetime_jst,
                result_utc = %schedule_datetime_utc,
                in_event_period = schedule_datetime_utc >= start_at_utc && schedule_datetime_utc <= end_at_utc,
                "スケジュール日時を計算しました（JST→UTC）"
            );
        }

        Ok(results)
    }

    /// start_day_relative文字列をパースして日付オフセットのリストを返す
    /// - "*" または "all": イベント期間中の全日
    /// - "start": 開始日のみ（オフセット0）
    /// - "end": 終了日のみ
    /// - "0-30" または "0 to 30": 範囲指定（0日目から30日目まで）
    /// - "5" または "-1": 単一の日数
    fn parse_start_day_relative(
        &self,
        relative_day: &str,
        start_at: chrono::NaiveDateTime,
        end_at: chrono::NaiveDateTime,
    ) -> Result<Vec<i64>> {
        let trimmed = relative_day.trim();

        // "*" または "all": 全日
        if trimmed == "*" || trimmed == "all" {
            let diff_days = (end_at - start_at).num_days();
            return Ok((0..diff_days).collect());
        }

        // "start": 開始日
        if trimmed == "start" {
            return Ok(vec![0]);
        }

        // "end": 終了日
        if trimmed == "end" {
            let diff_days = (end_at - start_at).num_days();
            return Ok(vec![diff_days]);
        }

        // 範囲指定のパターン: "0-30" または "0 to 30"
        // 正規表現: r'^(-?\d+)\s*(?:-|to)\s*(-?\d+)$'
        if let Some((start_str, end_str)) = self.parse_range(trimmed) {
            let start_day: i64 = start_str.parse().map_err(|e| {
                error!(
                    relative_day = %relative_day,
                    error = %e,
                    "範囲指定の開始日のパースに失敗しました"
                );
                crate::types::AppError::Validation {
                    field: format!("範囲指定の開始日: {start_str}"),
                }
            })?;

            let end_day: i64 = end_str.parse().map_err(|e| {
                error!(
                    relative_day = %relative_day,
                    error = %e,
                    "範囲指定の終了日のパースに失敗しました"
                );
                crate::types::AppError::Validation {
                    field: format!("範囲指定の終了日: {end_str}"),
                }
            })?;

            return Ok((start_day..=end_day).collect());
        }

        // 単一の数値: "5" または "-1"
        if let Ok(day) = trimmed.parse::<i64>() {
            return Ok(vec![day]);
        }

        error!(
            relative_day = %relative_day,
            "start_day_relativeの形式が不正です"
        );
        Err(crate::types::AppError::Validation {
            field: format!("start_day_relative: {relative_day}"),
        })
    }

    /// 範囲指定文字列から開始と終了を抽出
    /// 例: "0-30" -> Some(("0", "30"))
    /// 例: "0 to 30" -> Some(("0", "30"))
    fn parse_range<'a>(&self, s: &'a str) -> Option<(&'a str, &'a str)> {
        // "-" で分割を試みる（"-1-5" のような負の数を含む場合に対応）
        if let Some(pos) = s.rfind('-') {
            // 最後の "-" の位置を探す
            if pos > 0 {
                // 先頭が "-" でない場合
                let start = s[..pos].trim();
                let end = s[pos + 1..].trim();
                // 両方が数値として有効かチェック
                if start.parse::<i64>().is_ok() && end.parse::<i64>().is_ok() {
                    return Some((start, end));
                }
            }
        }

        // " to " で分割を試みる
        if let Some(pos) = s.find(" to ") {
            let start = s[..pos].trim();
            let end = s[pos + 4..].trim();
            return Some((start, end));
        }

        None
    }

    /// 時刻文字列をパース
    fn parse_time(&self, time_str: &str) -> Result<NaiveTime> {
        NaiveTime::parse_from_str(time_str, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M"))
            .map_err(|e| {
                error!(
                    time_str = %time_str,
                    error = %e,
                    "時刻のパースに失敗しました"
                );
                crate::types::AppError::Validation {
                    field: format!("時刻形式: {time_str}"),
                }
            })
    }
}

impl Default for ScheduleCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_start_day_relative_single() {
        let calculator = ScheduleCalculator::new();
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        assert_eq!(
            calculator
                .parse_start_day_relative("0", start, end)
                .unwrap(),
            vec![0]
        );
        assert_eq!(
            calculator
                .parse_start_day_relative("+1", start, end)
                .unwrap(),
            vec![1]
        );
        assert_eq!(
            calculator
                .parse_start_day_relative("-1", start, end)
                .unwrap(),
            vec![-1]
        );
        assert_eq!(
            calculator
                .parse_start_day_relative("5", start, end)
                .unwrap(),
            vec![5]
        );
    }

    #[test]
    fn test_parse_start_day_relative_range() {
        let calculator = ScheduleCalculator::new();
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 2, 14)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // "0-30" の範囲指定
        let result = calculator
            .parse_start_day_relative("0-30", start, end)
            .unwrap();
        assert_eq!(result.len(), 31); // 0から30まで（両端含む）
        assert_eq!(result[0], 0);
        assert_eq!(result[30], 30);

        // "0 to 5" の範囲指定
        let result = calculator
            .parse_start_day_relative("0 to 5", start, end)
            .unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_start_day_relative_keywords() {
        let calculator = ScheduleCalculator::new();
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // "start"
        assert_eq!(
            calculator
                .parse_start_day_relative("start", start, end)
                .unwrap(),
            vec![0]
        );

        // "end"
        let end_offset = (end - start).num_days();
        assert_eq!(
            calculator
                .parse_start_day_relative("end", start, end)
                .unwrap(),
            vec![end_offset]
        );

        // "*" または "all"
        let all_days: Vec<i64> = (0..end_offset).collect();
        assert_eq!(
            calculator
                .parse_start_day_relative("*", start, end)
                .unwrap(),
            all_days
        );
        assert_eq!(
            calculator
                .parse_start_day_relative("all", start, end)
                .unwrap(),
            all_days
        );
    }

    #[test]
    fn test_parse_time() {
        let calculator = ScheduleCalculator::new();

        let time1 = calculator.parse_time("05:00:00").unwrap();
        assert_eq!(time1.format("%H").to_string(), "05");
        assert_eq!(time1.format("%M").to_string(), "00");

        let time2 = calculator.parse_time("23:59:59").unwrap();
        assert_eq!(time2.format("%H").to_string(), "23");
        assert_eq!(time2.format("%M").to_string(), "59");

        let time3 = calculator.parse_time("12:30").unwrap();
        assert_eq!(time3.format("%H").to_string(), "12");
        assert_eq!(time3.format("%M").to_string(), "30");
    }

    #[test]
    fn test_calculate_datetimes_single() {
        let calculator = ScheduleCalculator::new();

        let event_schedule = event_schedules::Model {
            id: uuid::Uuid::new_v4(),
            event_type: "test".to_string(),
            event_count: 1,
            profile: "test_profile".to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let detail = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "0".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test".to_string(),
            message_text_id: "msg1".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let results = calculator
            .calculate_datetimes(&event_schedule, &detail)
            .unwrap();

        assert_eq!(results.len(), 1);
        let result = results[0];

        // JST 2025-01-15 05:00 → UTC 2025-01-14 20:00（JSTはUTC+9時間）
        assert_eq!(result.format("%Y").to_string(), "2025");
        assert_eq!(result.format("%m").to_string(), "01");
        assert_eq!(result.format("%d").to_string(), "14");
        assert_eq!(result.format("%H").to_string(), "20");
        assert_eq!(result.format("%M").to_string(), "00");
    }

    #[test]
    fn test_calculate_datetimes_range() {
        let calculator = ScheduleCalculator::new();

        let event_schedule = event_schedules::Model {
            id: uuid::Uuid::new_v4(),
            event_type: "test".to_string(),
            event_count: 1,
            profile: "test_profile".to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let detail = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "0-2".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test".to_string(),
            message_text_id: "msg1".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let results = calculator
            .calculate_datetimes(&event_schedule, &detail)
            .unwrap();

        // 0-2 の範囲指定なので3日分（0日目、1日目、2日目）
        assert_eq!(results.len(), 3);

        // 0日目: JST 2025-01-15 05:00 → UTC 2025-01-14 20:00
        assert_eq!(
            results[0].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-14 20:00"
        );
        // 1日目: JST 2025-01-16 05:00 → UTC 2025-01-15 20:00
        assert_eq!(
            results[1].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-15 20:00"
        );
        // 2日目: JST 2025-01-17 05:00 → UTC 2025-01-16 20:00
        assert_eq!(
            results[2].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-16 20:00"
        );
    }

    #[test]
    fn test_calculate_datetimes_before_start() {
        let calculator = ScheduleCalculator::new();

        let event_schedule = event_schedules::Model {
            id: uuid::Uuid::new_v4(),
            event_type: "test".to_string(),
            event_count: 1,
            profile: "test_profile".to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let detail = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "-1".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test_before".to_string(),
            message_text_id: "msg_before".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let results = calculator
            .calculate_datetimes(&event_schedule, &detail)
            .unwrap();

        // 開始日の1日前のスケジュールが作成されること
        assert_eq!(results.len(), 1);

        // JST 2025-01-14 05:00 → UTC 2025-01-13 20:00（開始日の1日前）
        assert_eq!(
            results[0].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-13 20:00"
        );
    }

    #[test]
    fn test_calculate_datetimes_after_end() {
        let calculator = ScheduleCalculator::new();

        let event_schedule = event_schedules::Model {
            id: uuid::Uuid::new_v4(),
            event_type: "test".to_string(),
            event_count: 1,
            profile: "test_profile".to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let detail = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "10".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test_after".to_string(),
            message_text_id: "msg_after".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let results = calculator
            .calculate_datetimes(&event_schedule, &detail)
            .unwrap();

        // 終了日後のスケジュールが作成されること
        // イベント期間は1/15-1/20（5日間）、10日目は1/25なので終了日後
        assert_eq!(results.len(), 1);

        // JST 2025-01-25 05:00 → UTC 2025-01-24 20:00（開始日の10日後）
        assert_eq!(
            results[0].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-24 20:00"
        );
    }

    #[test]
    fn test_calculate_datetimes_range_including_before_and_after() {
        let calculator = ScheduleCalculator::new();

        let event_schedule = event_schedules::Model {
            id: uuid::Uuid::new_v4(),
            event_type: "test".to_string(),
            event_count: 1,
            profile: "test_profile".to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 17)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let detail = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "-1-3".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test_range".to_string(),
            message_text_id: "msg_range".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let results = calculator
            .calculate_datetimes(&event_schedule, &detail)
            .unwrap();

        // -1日目から3日目まで（5日分）のスケジュールが作成されること
        // イベント期間は1/15-1/17（2日間）
        // -1日目: 1/14（開始前）
        // 0日目: 1/15（期間内）
        // 1日目: 1/16（期間内）
        // 2日目: 1/17（期間内・終了日）
        // 3日目: 1/18（終了後）
        assert_eq!(results.len(), 5);

        assert_eq!(
            results[0].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-13 20:00" // -1日目
        );
        assert_eq!(
            results[1].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-14 20:00" // 0日目
        );
        assert_eq!(
            results[2].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-15 20:00" // 1日目
        );
        assert_eq!(
            results[3].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-16 20:00" // 2日目
        );
        assert_eq!(
            results[4].format("%Y-%m-%d %H:%M").to_string(),
            "2025-01-17 20:00" // 3日目
        );
    }
}
