use crate::models::entities::{event_schedule_details, event_schedules};
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
                let guild_channels = match guild_channels_by_type.get(&detail.notification_channel_type) {
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
                    match self.calculate_datetime(event_schedule, detail) {
                        Ok(schedule_datetime) => {
                            results.push(CalculatedSchedule {
                                schedule_datetime,
                                guild_id: *guild_id,
                                channel_id: *channel_id,
                                message_text_id: detail.message_text_id.clone(),
                                event_schedule_id: event_schedule.id,
                                event_schedule_detail_id: detail.id,
                            });
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

    /// イベントスケジュールと詳細から具体的な日時を計算
    fn calculate_datetime(
        &self,
        event_schedule: &event_schedules::Model,
        detail: &event_schedule_details::Model,
    ) -> Result<DateTime<Utc>> {
        // start_day_relativeをパース（例: "+0", "+1", "-1"）
        let day_offset = self.parse_day_offset(&detail.start_day_relative)?;

        // timeをパース（例: "05:00:00", "23:59:59"）
        let time = self.parse_time(&detail.time)?;

        // イベント開始日時に日数オフセットを追加（JSTのまま計算）
        let target_datetime_jst = event_schedule.start_at + Duration::days(day_offset);

        // 日付と時刻を組み合わせる（JSTとして）
        let naive_datetime = target_datetime_jst
            .date()
            .and_time(time);

        // JSTをUTCに変換（JST = UTC+9）
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let schedule_datetime_jst = jst
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or_else(|| crate::types::AppError::Validation {
                field: format!("日時計算: {}", naive_datetime),
            })?;

        let schedule_datetime_utc = schedule_datetime_jst.with_timezone(&Utc);

        debug!(
            event_start_jst = %event_schedule.start_at,
            day_offset = day_offset,
            time = %detail.time,
            result_jst = %schedule_datetime_jst,
            result_utc = %schedule_datetime_utc,
            "スケジュール日時を計算しました（JST→UTC）"
        );

        Ok(schedule_datetime_utc)
    }

    /// 日付オフセット文字列をパース
    fn parse_day_offset(&self, offset_str: &str) -> Result<i64> {
        offset_str
            .parse::<i64>()
            .map_err(|e| {
                error!(
                    offset_str = %offset_str,
                    error = %e,
                    "日付オフセットのパースに失敗しました"
                );
                crate::types::AppError::Validation {
                    field: format!("日付オフセット: {}", offset_str),
                }
            })
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
                    field: format!("時刻形式: {}", time_str),
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
    fn test_parse_day_offset() {
        let calculator = ScheduleCalculator::new();

        assert_eq!(calculator.parse_day_offset("0").unwrap(), 0);
        assert_eq!(calculator.parse_day_offset("+1").unwrap(), 1);
        assert_eq!(calculator.parse_day_offset("-1").unwrap(), -1);
        assert_eq!(calculator.parse_day_offset("5").unwrap(), 5);
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
    fn test_calculate_datetime() {
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

        let result = calculator.calculate_datetime(&event_schedule, &detail).unwrap();

        // JST 2025-01-15 05:00 → UTC 2025-01-14 20:00（JSTはUTC+9時間）
        assert_eq!(result.format("%Y").to_string(), "2025");
        assert_eq!(result.format("%m").to_string(), "01");
        assert_eq!(result.format("%d").to_string(), "14");
        assert_eq!(result.format("%H").to_string(), "20");
        assert_eq!(result.format("%M").to_string(), "00");
    }
}
