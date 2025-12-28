use crate::services::datetime_parser::parse_event_date;
use crate::types::{AppError, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;
use tracing::{debug, warn};

lazy_static! {
    /// 相対時刻パターン（日本語・英語・数字+単位）
    /// 例: 1日前, 1日, 1day, 1days, 1時間前, 1h, 90分前, 90m
    static ref RE_RELATIVE_TIME: Regex = Regex::new(
        r"(?x)
        ^
        (\d+)\s*                    # 数値（スペース許可）
        (日|日前|days?|時間?|時間前|hours?|h|分|分前|minutes?|mins?|m) # 単位
        (?:前)?                      # オプショナルな「前」
        $
        "
    )
    .expect("相対時刻Regexパターンが無効です");
}

/// パース済み解散時刻
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDismissalTime {
    /// 絶対日時
    Absolute {
        input_value: String,
        datetime: DateTime<Utc>,
    },
    /// 相対時刻
    Relative {
        input_value: String,
        days: i32,
        hours: i32,
        minutes: i32,
    },
}

/// 解散時刻パーサーサービス
///
/// **非推奨**: このサービスは `unified_datetime_parser` に統合されました。
/// 代わりに `DateTimeParseOptions::for_dismissal_time()` を使用してください。
#[deprecated(
    since = "0.5.0",
    note = "代わりに unified_datetime_parser::DateTimeParseOptions::for_dismissal_time() を使用してください"
)]
pub struct DismissalTimeParserService;

impl DismissalTimeParserService {
    pub fn new() -> Self {
        Self
    }

    /// 解散時刻文字列をパース
    ///
    /// # 引数
    /// - `input`: カンマ区切りの解散時刻文字列（最大3つ）
    /// - `departure_time`: 出発日時
    /// - `timezone`: タイムゾーン
    /// - `max_days`: 最大指定可能日数（デフォルト7日）
    ///
    /// # 戻り値
    /// パース済み解散時刻のベクタ
    pub fn parse(
        &self,
        input: &str,
        departure_time: DateTime<Utc>,
        timezone: Tz,
        max_days: i32,
    ) -> Result<Vec<ParsedDismissalTime>> {
        debug!(
            input,
            departure_time = %departure_time,
            timezone = %timezone,
            max_days,
            "解散時刻のパースを開始します"
        );

        // 1. カンマで分割
        let parts: Vec<&str> = input
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // 2. 4つ以上あればエラー
        if parts.len() > 3 {
            return Err(AppError::Business {
                message: format!(
                    "解散時刻は最大3つまで指定できます（指定された数: {}）",
                    parts.len()
                ),
            });
        }

        // 3. 各要素をパース
        let mut results = Vec::new();
        for part in parts {
            let parsed = self.parse_single(part, departure_time, timezone)?;
            results.push(parsed);
        }

        // 4. 最大日数チェック
        let max_duration = Duration::days(max_days as i64);
        for parsed in &results {
            let dismissal_datetime = match parsed {
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

            // 出発時刻より後の場合はエラー
            if dismissal_datetime >= departure_time {
                return Err(AppError::Business {
                    message: format!(
                        "解散時刻は出発時刻（{}）より前である必要があります",
                        departure_time
                    ),
                });
            }

            // 最大日数を超える場合はエラー
            let diff = departure_time - dismissal_datetime;
            if diff > max_duration {
                return Err(AppError::Business {
                    message: format!(
                        "解散時刻は出発時刻の{}日前までしか指定できません（指定: {}）",
                        max_days,
                        match parsed {
                            ParsedDismissalTime::Absolute { input_value, .. } => input_value,
                            ParsedDismissalTime::Relative { input_value, .. } => input_value,
                        }
                    ),
                });
            }
        }

        debug!(count = results.len(), "解散時刻のパースが完了しました");
        Ok(results)
    }

    /// 単一の解散時刻文字列をパース
    fn parse_single(
        &self,
        input: &str,
        departure_time: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<ParsedDismissalTime> {
        // 1. 相対時刻パターンを試行
        if let Some(caps) = RE_RELATIVE_TIME.captures(input) {
            let value = caps[1].parse::<i32>().map_err(|_| AppError::Business {
                message: format!("数値のパースに失敗しました: {}", &caps[1]),
            })?;
            let unit = &caps[2];

            let (days, hours, minutes) = match unit {
                "日" | "日前" | "day" | "days" => (value, 0, 0),
                "時" | "時間" | "時間前" | "hour" | "hours" | "h" => (0, value, 0),
                "分" | "分前" | "minute" | "minutes" | "min" | "mins" | "m" => (0, 0, value),
                _ => {
                    return Err(AppError::Business {
                        message: format!("不明な時間単位です: {}", unit),
                    });
                }
            };

            debug!(input, days, hours, minutes, "相対時刻としてパースしました");

            return Ok(ParsedDismissalTime::Relative {
                input_value: input.to_string(),
                days,
                hours,
                minutes,
            });
        }

        // 2. 絶対日時パターンを試行
        // parse_event_dateは「今日」を基準にするため使用しない
        // 代わりに、出発日時の日付を基準にして解散時刻を構築する
        self.parse_absolute_time(input, departure_time, timezone)
    }

    /// 絶対時刻をパースして出発日の日付と組み合わせる
    fn parse_absolute_time(
        &self,
        input: &str,
        departure_time: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<ParsedDismissalTime> {
        use chrono::NaiveTime;

        // 出発日時をローカルタイムゾーンに変換
        let departure_local = departure_time.with_timezone(&timezone);

        // 時刻のパース（複数フォーマット対応）
        let parsed_time = if let Ok(t) = NaiveTime::parse_from_str(input, "%H:%M") {
            t
        } else if let Ok(t) = NaiveTime::parse_from_str(input, "%-H:%M") {
            t
        } else if let Ok(t) = NaiveTime::parse_from_str(input, "%H時%M分") {
            t
        } else {
            // 上記以外のフォーマットはparse_event_dateに委譲
            match parse_event_date(input, timezone) {
                Ok(parsed_datetime) => parsed_datetime.time(),
                Err(e) => {
                    warn!(input, error = %e, "解散時刻のパースに失敗しました");
                    return Err(AppError::Business {
                        message: format!("解散時刻の形式が正しくありません: {}", input),
                    });
                }
            }
        };

        // 出発日の同じ時刻で日時を作成
        let mut dismissal_datetime_local = timezone
            .from_local_datetime(&departure_local.date_naive().and_time(parsed_time))
            .single()
            .ok_or_else(|| AppError::Business {
                message: "解散時刻の日時変換に失敗しました".to_string(),
            })?;

        // 出発時刻より後になる場合は前日にする
        if dismissal_datetime_local >= departure_local {
            dismissal_datetime_local = dismissal_datetime_local - Duration::days(1);
        }

        let dismissal_datetime_utc = dismissal_datetime_local.with_timezone(&Utc);

        debug!(
            input,
            departure_time = %departure_time,
            departure_local = %departure_local,
            parsed_time = %parsed_time,
            dismissal_datetime_local = %dismissal_datetime_local,
            dismissal_datetime_utc = %dismissal_datetime_utc,
            "絶対日時としてパースしました"
        );

        Ok(ParsedDismissalTime::Absolute {
            input_value: input.to_string(),
            datetime: dismissal_datetime_utc,
        })
    }
}

impl Default for DismissalTimeParserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    #[test]
    fn test_parse_relative_time_days() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 1日前
        let result = service.parse("1日前", departure, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            ParsedDismissalTime::Relative {
                days,
                hours,
                minutes,
                ..
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 0);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }

        // 1day
        let result = service.parse("1day", departure, tz, 7);
        assert!(result.is_ok());

        // 3 days
        let result = service.parse("3 days", departure, tz, 7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_relative_time_hours() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 1時間前
        let result = service.parse("1時間前", departure, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            ParsedDismissalTime::Relative {
                days,
                hours,
                minutes,
                ..
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 1);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }

        // 1h
        let result = service.parse("1h", departure, tz, 7);
        assert!(result.is_ok());

        // 12 hours
        let result = service.parse("12 hours", departure, tz, 7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_relative_time_minutes() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 90分前
        let result = service.parse("90分前", departure, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            ParsedDismissalTime::Relative {
                days,
                hours,
                minutes,
                ..
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 0);
                assert_eq!(*minutes, 90);
            }
            _ => panic!("Expected Relative"),
        }

        // 30m
        let result = service.parse("30m", departure, tz, 7);
        assert!(result.is_ok());

        // 120 minutes
        let result = service.parse("120 minutes", departure, tz, 7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_absolute_time() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 21:00
        let result = service.parse("21:00", departure, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(matches!(parsed[0], ParsedDismissalTime::Absolute { .. }));
    }

    #[test]
    fn test_parse_multiple() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // カンマ区切り3つ
        let result = service.parse("1時間前, 21:00, 2日前", departure, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn test_parse_too_many() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 4つ以上
        let result = service.parse("1時間前, 21:00, 2日前, 3日前", departure, tz, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_exceeds_max_days() {
        let service = DismissalTimeParserService::new();
        let departure = Utc.with_ymd_and_hms(2025, 12, 26, 22, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 8日前（max_days=7を超える）
        let result = service.parse("8日前", departure, tz, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_absolute_time_with_timezone_basic() {
        let service = DismissalTimeParserService::new();
        // JST 2025-12-26 08:20 = UTC 2025-12-25 23:20
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 23, 20, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 8:10（出発10分前、JSTで指定）
        let result = service.parse("8:10", departure_utc, tz, 7);
        assert!(result.is_ok(), "Parse should succeed");
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-25 23:10 (= JST 2025-12-26 08:10) であることを確認
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 23);
                assert_eq!(datetime.minute(), 10);

                // 出発時刻より前であることを確認
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_jst_no_date_change() {
        // パターン1: JST（UTC変換で日付が変わらない）
        // JST 2025-12-26 15:00 = UTC 2025-12-26 06:00
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 26, 6, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 14:00（出発1時間前、JSTで指定）
        let result = service.parse("14:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-26 05:00 (= JST 2025-12-26 14:00)
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 26);
                assert_eq!(datetime.hour(), 5);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_utc_only() {
        // パターン2: UTCのみ
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 26, 10, 0, 0).unwrap();
        let tz = chrono_tz::UTC;

        // 9:00（出発1時間前、UTCで指定）
        let result = service.parse("9:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-26 09:00
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 26);
                assert_eq!(datetime.hour(), 9);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_jst_departure_date_changes() {
        // パターン3: JSTから変換で開催日が変化
        // JST 2025-12-26 02:00 = UTC 2025-12-25 17:00
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 17, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 1:00（出発1時間前、JSTで指定）
        let result = service.parse("1:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-25 16:00 (= JST 2025-12-26 01:00)
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 16);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_jst_dismissal_date_changes() {
        // パターン4: JSTから変換で解散日が変化（前日調整により）
        // JST 2025-12-26 01:00 = UTC 2025-12-25 16:00
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 16, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 23:00（出発より後なので前日の23:00になる、JSTで指定）
        let result = service.parse("23:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // 前日調整により JST 2025-12-25 23:00 = UTC 2025-12-25 14:00
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 14);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_jst_both_dates_change() {
        // パターン5: JSTから変換で開催日と解散日の両方が変化
        // JST 2025-12-26 00:30 = UTC 2025-12-25 15:30
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 15, 30, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 22:00（出発より後なので前日の22:00になる、JSTで指定）
        let result = service.parse("22:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // 前日調整により JST 2025-12-25 22:00 = UTC 2025-12-25 13:00
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 13);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_jst_edge_case_midnight() {
        // エッジケース: 深夜0時付近
        // JST 2025-12-26 00:00 = UTC 2025-12-25 15:00
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 15, 0, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 23:30（出発より後なので前日の23:30になる）
        let result = service.parse("23:30", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // 前日調整により JST 2025-12-25 23:30 = UTC 2025-12-25 14:30
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 14);
                assert_eq!(datetime.minute(), 30);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_mixed_relative_and_absolute() {
        // 混合パターン: 相対時刻と絶対時刻の組み合わせ
        // JST 2025-12-26 08:20 = UTC 2025-12-25 23:20
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 25, 23, 20, 0).unwrap();
        let tz = chrono_tz::Asia::Tokyo;

        // 相対時刻（1時間前）と絶対時刻（8:10）の混合
        let result = service.parse("1時間前, 8:10", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 2);

        // 1つ目: 相対時刻
        match &parsed[0] {
            ParsedDismissalTime::Relative {
                days,
                hours,
                minutes,
                ..
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 1);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }

        // 2つ目: 絶対時刻
        match &parsed[1] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-25 23:10 (= JST 2025-12-26 08:10)
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 25);
                assert_eq!(datetime.hour(), 23);
                assert_eq!(datetime.minute(), 10);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_timezone_pst_to_utc() {
        // 別のタイムゾーン（太平洋標準時）でのテスト
        // PST 2025-12-25 20:00 = UTC 2025-12-26 04:00
        let service = DismissalTimeParserService::new();
        let departure_utc = Utc.with_ymd_and_hms(2025, 12, 26, 4, 0, 0).unwrap();
        let tz = chrono_tz::America::Los_Angeles;

        // 19:00（出発1時間前、PSTで指定）
        let result = service.parse("19:00", departure_utc, tz, 7);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        match &parsed[0] {
            ParsedDismissalTime::Absolute { datetime, .. } => {
                // UTC 2025-12-26 03:00 (= PST 2025-12-25 19:00)
                assert_eq!(datetime.year(), 2025);
                assert_eq!(datetime.month(), 12);
                assert_eq!(datetime.day(), 26);
                assert_eq!(datetime.hour(), 3);
                assert_eq!(datetime.minute(), 0);
                assert!(datetime < &departure_utc);
            }
            _ => panic!("Expected Absolute"),
        }
    }
}
