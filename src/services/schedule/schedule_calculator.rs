use crate::models::entities::master::{event_schedule_details, event_schedules};
use crate::types::{AppError, Result};
use chrono::{DateTime, Duration, FixedOffset, NaiveTime, TimeZone, Utc};
use std::collections::HashMap;
use tracing::{debug, error, warn};

/// スケジュール計算サービス
pub struct ScheduleCalculator {
    /// イベント期間外のスケジュール作成を許可する最大日数
    max_schedule_days_outside_event: i64,
}

/// スケジュール計算結果
#[derive(Debug, Clone)]
pub struct CalculatedSchedule {
    pub schedule_datetime: DateTime<Utc>,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_text_id: String,
    pub event_schedule_id: Option<uuid::Uuid>,
    pub event_schedule_detail_id: Option<uuid::Uuid>,
}

impl ScheduleCalculator {
    pub fn new(max_schedule_days_outside_event: i64) -> Self {
        Self {
            max_schedule_days_outside_event,
        }
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
                                    event_schedule_id: Some(event_schedule.id),
                                    event_schedule_detail_id: Some(detail.id),
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
    pub(crate) fn calculate_datetimes(
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
        let jst = FixedOffset::east_opt(9 * 3600).ok_or_else(|| AppError::Config {
            message: "JSTオフセットの生成に失敗しました".to_string(),
        })?;

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
    /// - "0-30" または "0 to 30" または "0to30": 範囲指定（0日目から30日目まで）
    /// - "5" または "-1": 単一の日数
    ///
    /// # 範囲外チェック
    /// イベント期間外（開始日前・終了日後）のスケジュールは、
    /// max_schedule_days_outside_eventで設定された日数以内のみ許可されます。
    fn parse_start_day_relative(
        &self,
        relative_day: &str,
        start_at: chrono::NaiveDateTime,
        end_at: chrono::NaiveDateTime,
    ) -> Result<Vec<i64>> {
        let trimmed = relative_day.trim();
        let event_duration_days = (end_at - start_at).num_days();

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

            let days: Vec<i64> = (start_day..=end_day).collect();
            self.validate_days_range(&days, event_duration_days)?;
            return Ok(days);
        }

        // 単一の数値: "5" または "-1"
        if let Ok(day) = trimmed.parse::<i64>() {
            let days = vec![day];
            self.validate_days_range(&days, event_duration_days)?;
            return Ok(days);
        }

        error!(
            relative_day = %relative_day,
            "start_day_relativeの形式が不正です"
        );
        Err(crate::types::AppError::Validation {
            field: format!("start_day_relative: {relative_day}"),
        })
    }

    /// 日数オフセットのリストがイベント期間外の制限範囲内かチェック
    ///
    /// # Arguments
    /// * `days` - 日数オフセットのリスト
    /// * `event_duration_days` - イベント期間の日数（終了日 - 開始日）
    ///
    /// # Errors
    /// イベント期間外のオフセットがmax_schedule_days_outside_eventを超える場合、エラーを返す
    fn validate_days_range(&self, days: &[i64], event_duration_days: i64) -> Result<()> {
        for &day in days {
            // 開始日前のチェック（負の値）
            if day < 0 && day.abs() > self.max_schedule_days_outside_event {
                error!(
                    day_offset = day,
                    max_days = self.max_schedule_days_outside_event,
                    "イベント開始日前のスケジュールが許可範囲を超えています"
                );
                return Err(crate::types::AppError::Validation {
                    field: format!(
                        "開始日の{}日前のスケジュールは許可されていません（最大: {}日前まで）",
                        day.abs(),
                        self.max_schedule_days_outside_event
                    ),
                });
            }

            // 終了日後のチェック
            if day > event_duration_days {
                let days_after_end = day - event_duration_days;
                if days_after_end > self.max_schedule_days_outside_event {
                    error!(
                        day_offset = day,
                        event_duration = event_duration_days,
                        days_after_end = days_after_end,
                        max_days = self.max_schedule_days_outside_event,
                        "イベント終了日後のスケジュールが許可範囲を超えています"
                    );
                    return Err(crate::types::AppError::Validation {
                        field: format!(
                            "終了日の{}日後のスケジュールは許可されていません（最大: {}日後まで）",
                            days_after_end, self.max_schedule_days_outside_event
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    /// 範囲指定文字列から開始と終了を抽出
    /// 例: "0-30" -> Some(("0", "30"))
    /// 例: "0 to 30" -> Some(("0", "30"))
    /// 例: "0to30" -> Some(("0", "30"))
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

        // "to" で分割を試みる（スペースあり・なし両対応）
        // 正規表現 \d+to\d+ にマッチする位置を探す
        if let Some(pos) = s.find("to") {
            let start = s[..pos].trim();
            let end = s[pos + 2..].trim();
            if start.parse::<i64>().is_ok() && end.parse::<i64>().is_ok() {
                return Some((start, end));
            }
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
        Self::new(365) // デフォルトは365日
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_start_day_relative_single() {
        let calculator = ScheduleCalculator::new(365);
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
        let calculator = ScheduleCalculator::new(365);
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

        // "0 to 5" の範囲指定（スペースあり）
        let result = calculator
            .parse_start_day_relative("0 to 5", start, end)
            .unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);

        // "0to6" の範囲指定（スペースなし）
        let result = calculator
            .parse_start_day_relative("0to6", start, end)
            .unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_parse_start_day_relative_keywords() {
        let calculator = ScheduleCalculator::new(365);
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
        let calculator = ScheduleCalculator::new(365);

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
        let calculator = ScheduleCalculator::new(365);

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
        let calculator = ScheduleCalculator::new(365);

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
        let calculator = ScheduleCalculator::new(365);

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
        let calculator = ScheduleCalculator::new(365);

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
        let calculator = ScheduleCalculator::new(365);

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

    #[test]
    fn test_validate_days_range_within_limit() {
        // 最大31日の制限でテスト
        let calculator = ScheduleCalculator::new(31);
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // 開始日の31日前は許可される
        let result = calculator.parse_start_day_relative("-31", start, end);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![-31]);

        // 終了日の31日後は許可される（開始日から36日後）
        let result = calculator.parse_start_day_relative("36", start, end);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![36]);

        // 期間内は当然許可される
        let result = calculator.parse_start_day_relative("0-4", start, end);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[test]
    fn test_validate_days_range_exceeds_limit_before_start() {
        // 最大31日の制限でテスト
        let calculator = ScheduleCalculator::new(31);
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // 開始日の32日前は拒否される
        let result = calculator.parse_start_day_relative("-32", start, end);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("32日前"));
        assert!(err_msg.contains("31日前まで"));
    }

    #[test]
    fn test_validate_days_range_exceeds_limit_after_end() {
        // 最大31日の制限でテスト
        let calculator = ScheduleCalculator::new(31);
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // イベント期間は5日間（0-4日目）
        // 終了日の32日後は拒否される（開始日から37日後）
        let result = calculator.parse_start_day_relative("37", start, end);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("32日後"));
        assert!(err_msg.contains("31日後まで"));
    }

    #[test]
    fn test_validate_days_range_with_range_specification() {
        // 最大10日の制限でテスト
        let calculator = ScheduleCalculator::new(10);
        let start = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // -10から15の範囲（終了日は4日目なので、15日目は終了日の11日後）
        // 11日後は制限を超えるため拒否される
        let result = calculator.parse_start_day_relative("-10-16", start, end);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("11日後"));

        // -10から15の範囲（終了日の10日後まで）は許可される
        let result = calculator.parse_start_day_relative("-10-15", start, end);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 26); // -10から15まで
    }

    #[test]
    fn test_calculate_datetimes_exceeds_limit() {
        // 最大5日の制限でテスト
        let calculator = ScheduleCalculator::new(5);

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

        // 開始日の10日前（制限超過）
        let detail_before = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "-10".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "test_before".to_string(),
            message_text_id: "msg_before".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = calculator.calculate_datetimes(&event_schedule, &detail_before);
        assert!(result.is_err());

        // 終了日の10日後（制限超過）
        let detail_after = event_schedule_details::Model {
            id: uuid::Uuid::new_v4(),
            profile: "test_profile".to_string(),
            start_day_relative: "15".to_string(), // 終了日（4日目）の11日後
            time: "05:00:00".to_string(),
            schedule_name: "test_after".to_string(),
            message_text_id: "msg_after".to_string(),
            notification_channel_type: 1,
            reactions: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = calculator.calculate_datetimes(&event_schedule, &detail_after);
        assert!(result.is_err());
    }
}
