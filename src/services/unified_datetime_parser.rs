/// 統一日時パーサー
///
/// ビットフラグベースの柔軟な日時解析システム。
/// 既存の複数のパーサー（datetime_parser, TimeParserService, DismissalTimeParserService）を統合。
use crate::types::Result;
use chrono::{DateTime, Datelike, NaiveTime, Utc};
use chrono_tz::Tz;

/// 日時解析パターンフラグ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeParseFlags {
    bits: u32,
}

impl DateTimeParseFlags {
    /// 完全日時: "2025/11/15 21:00", "2025-11-15 21:00"
    pub const FULL_DATETIME: Self = Self { bits: 0b00000001 };

    /// 年なし日時: "12/11 14:00", "12-11 14:00"
    pub const DATETIME_NO_YEAR: Self = Self { bits: 0b00000010 };

    /// 日付のみ: "11/15", "11-15"
    pub const DATE_ONLY: Self = Self { bits: 0b00000100 };

    /// 時刻のみ: "21:00", "21時"
    pub const TIME_ONLY: Self = Self { bits: 0b00001000 };

    /// 日本語日時: "1月2日3時4分", "午後9時半"
    pub const JAPANESE_DATETIME: Self = Self { bits: 0b00010000 };

    /// 数字パターン: "1230", "10111230", "30 1230"
    pub const NUMERIC_PATTERNS: Self = Self { bits: 0b00100000 };

    /// 相対時刻: "2時間前", "1day", "90分前"
    pub const RELATIVE_TIME: Self = Self { bits: 0b01000000 };

    /// すべてのパターンを許可
    pub const ALL: Self = Self {
        bits: Self::FULL_DATETIME.bits
            | Self::DATETIME_NO_YEAR.bits
            | Self::DATE_ONLY.bits
            | Self::TIME_ONLY.bits
            | Self::JAPANESE_DATETIME.bits
            | Self::NUMERIC_PATTERNS.bits
            | Self::RELATIVE_TIME.bits,
    };

    /// 何も許可しない（空）
    pub const NONE: Self = Self { bits: 0 };

    /// フラグの結合
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// フラグの除外
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }

    /// フラグが含まれているか
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// フラグが空か
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

/// ビット演算のための実装
impl std::ops::BitOr for DateTimeParseFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for DateTimeParseFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::Not for DateTimeParseFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { bits: !self.bits }
    }
}

/// 相対時刻の基準
#[derive(Debug, Clone)]
pub enum RelativeBase {
    /// DateTime基準（解散時刻などで使用）
    DateTime(DateTime<Utc>),

    /// NaiveTime基準（定期募集開始時刻などで使用）
    Time(NaiveTime),
}

/// 日時解析オプション
#[derive(Debug, Clone)]
pub struct DateTimeParseOptions {
    /// 許可するパターンフラグ
    pub flags: DateTimeParseFlags,

    /// タイムゾーン
    pub timezone: Tz,

    /// 相対時刻の基準（RELATIVE_TIME有効時に必要）
    pub relative_base: Option<RelativeBase>,

    /// デフォルト時刻（DATE_ONLYで日付のみの場合に使用）
    pub default_time: Option<NaiveTime>,

    /// 複数時刻の許可（カンマ区切り）
    pub allow_multiple: bool,

    /// 最大個数（allow_multiple=true時）
    pub max_count: usize,
}

impl DateTimeParseOptions {
    /// クエスト出発日時用（絶対日時のみ、多様なパターン）
    ///
    /// # 対応パターン
    /// - 完全日時: "2025/11/15 21:00"
    /// - 年なし日時: "12/11 14:00"
    /// - 日付のみ: "11/15" (デフォルト21時)
    /// - 時刻のみ: "21:00"
    /// - 日本語: "1月2日3時4分", "午後9時半"
    /// - 数字: "1230", "10111230", "30 1230"
    pub fn for_quest_departure(timezone: Tz) -> Self {
        Self {
            flags: DateTimeParseFlags::ALL.difference(DateTimeParseFlags::RELATIVE_TIME),
            timezone,
            relative_base: None,
            default_time: Some(NaiveTime::from_hms_opt(21, 0, 0).unwrap()),
            allow_multiple: false,
            max_count: 1,
        }
    }

    /// 解散時刻用（相対・絶対両方、複数可、最大3つ）
    ///
    /// # 対応パターン
    /// - すべての絶対日時パターン
    /// - 相対時刻: "1時間前", "2日前", "90分前"
    /// - カンマ区切りで最大3つ
    pub fn for_dismissal_time(timezone: Tz, base_datetime: DateTime<Utc>) -> Self {
        Self {
            flags: DateTimeParseFlags::ALL,
            timezone,
            relative_base: Some(RelativeBase::DateTime(base_datetime)),
            default_time: None,
            allow_multiple: true,
            max_count: 3,
        }
    }

    /// 定期募集開始時刻用（時刻のみ + 相対時刻）
    ///
    /// # 対応パターン
    /// - 時刻: "21:00", "21時", "午後9時半"
    /// - 数字: "1230"
    /// - 相対時刻: "2時間前", "1h"（クエスト開始時刻を基準）
    pub fn for_schedule_start_time(timezone: Tz, base_time: NaiveTime) -> Self {
        Self {
            flags: DateTimeParseFlags::TIME_ONLY
                | DateTimeParseFlags::JAPANESE_DATETIME
                | DateTimeParseFlags::NUMERIC_PATTERNS
                | DateTimeParseFlags::RELATIVE_TIME,
            timezone,
            relative_base: Some(RelativeBase::Time(base_time)),
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        }
    }

    /// HH:MM厳格モード（既存TimeParserService互換）
    ///
    /// # 対応パターン
    /// - "22:00", "09:30" のみ
    pub fn strict_hhmm_only(timezone: Tz) -> Self {
        Self {
            flags: DateTimeParseFlags::NONE, // カスタムバリデーションで処理
            timezone,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        }
    }
}

/// 解析結果
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDateTime {
    /// 絶対日時
    Absolute(DateTime<Utc>),

    /// 相対時刻（基準時刻からのオフセット）
    Relative {
        days: i32,
        hours: i32,
        minutes: i32,
    },

    /// NaiveTime（定期募集開始時刻など）
    Time(NaiveTime),
}

/// 統一日時パーサー
///
/// # 引数
/// - `input`: 解析する文字列
/// - `options`: 解析オプション
///
/// # 戻り値
/// パース済み日時のベクタ（allow_multiple=falseの場合は要素数1）
///
/// # エラー
/// - パースに失敗した場合
/// - 最大個数を超えた場合
pub fn parse_datetime(input: &str, options: &DateTimeParseOptions) -> Result<Vec<ParsedDateTime>> {
    let trimmed = input.trim();

    // 複数指定の処理
    if options.allow_multiple {
        let parts: Vec<&str> = trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() > options.max_count {
            return Err(format!(
                "最大{}つまで指定できます（指定された数: {}）",
                options.max_count,
                parts.len()
            )
            .into());
        }

        let mut results = Vec::new();
        for part in parts {
            let parsed = parse_single(part, options)?;
            results.push(parsed);
        }

        Ok(results)
    } else {
        let parsed = parse_single(trimmed, options)?;
        Ok(vec![parsed])
    }
}

/// 単一の日時文字列をパース
fn parse_single(input: &str, options: &DateTimeParseOptions) -> Result<ParsedDateTime> {
    // HH:MM厳格モード
    if options.flags.is_empty() {
        return parse_strict_hhmm(input, options);
    }

    // 相対時刻を試行
    if options.flags.contains(DateTimeParseFlags::RELATIVE_TIME) {
        if let Ok(relative) = parse_relative_time(input, options) {
            return Ok(relative);
        }
    }

    // 既存のdatetime_parserを使用して絶対日時をパース
    // TODO: フラグに基づいて各パターンを個別に試行する実装
    // 現時点では既存のparse_event_dateを流用
    let dt = crate::services::datetime_parser::parse_event_date(input, options.timezone)?;

    Ok(ParsedDateTime::Absolute(dt))
}

/// HH:MM厳格モードのパース
fn parse_strict_hhmm(input: &str, _options: &DateTimeParseOptions) -> Result<ParsedDateTime> {
    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() != 2 {
        return Err(format!(
            "無効な時刻形式です: {input}（HH:MM形式で指定してください）"
        )
        .into());
    }

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("無効な時刻です: {input}"))?;

    let minute = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("無効な時刻です: {input}"))?;

    let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| format!("無効な時刻です: {input}"))?;

    Ok(ParsedDateTime::Time(naive_time))
}

/// 相対時刻のパース
fn parse_relative_time(
    input: &str,
    _options: &DateTimeParseOptions,
) -> Result<ParsedDateTime> {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
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

    if let Some(caps) = RE_RELATIVE_TIME.captures(input) {
        let value = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("数値のパースに失敗しました: {}", &caps[1]))?;
        let unit = &caps[2];

        let (days, hours, minutes) = match unit {
            "日" | "日前" | "day" | "days" => (value, 0, 0),
            "時" | "時間" | "時間前" | "hour" | "hours" | "h" => (0, value, 0),
            "分" | "分前" | "minute" | "minutes" | "min" | "mins" | "m" => (0, 0, value),
            _ => {
                return Err(format!("不明な時間単位です: {unit}").into());
            }
        };

        return Ok(ParsedDateTime::Relative {
            days,
            hours,
            minutes,
        });
    }

    Err("相対時刻のパースに失敗".to_string().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_flags_union() {
        let flags = DateTimeParseFlags::TIME_ONLY | DateTimeParseFlags::DATE_ONLY;
        assert!(flags.contains(DateTimeParseFlags::TIME_ONLY));
        assert!(flags.contains(DateTimeParseFlags::DATE_ONLY));
        assert!(!flags.contains(DateTimeParseFlags::FULL_DATETIME));
    }

    #[test]
    fn test_flags_difference() {
        let flags = DateTimeParseFlags::ALL.difference(DateTimeParseFlags::RELATIVE_TIME);
        assert!(flags.contains(DateTimeParseFlags::TIME_ONLY));
        assert!(!flags.contains(DateTimeParseFlags::RELATIVE_TIME));
    }

    #[test]
    fn test_parse_strict_hhmm() {
        let options = DateTimeParseOptions::strict_hhmm_only(chrono_tz::Asia::Tokyo);
        let result = parse_datetime("22:00", &options).unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            ParsedDateTime::Time(t) => {
                assert_eq!(t.hour(), 22);
                assert_eq!(t.minute(), 0);
            }
            _ => panic!("Expected Time"),
        }
    }

    #[test]
    fn test_parse_strict_hhmm_invalid() {
        let options = DateTimeParseOptions::strict_hhmm_only(chrono_tz::Asia::Tokyo);
        let result = parse_datetime("2200", &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_relative_time_hours() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        let result = parse_datetime("2時間前", &options).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_parse_relative_time_days() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        let result = parse_datetime("1day", &options).unwrap();

        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 0);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_parse_multiple() {
        let base = chrono::Utc::now();
        let options = DateTimeParseOptions::for_dismissal_time(chrono_tz::Asia::Tokyo, base);

        let result = parse_datetime("1時間前, 21:00, 2日前", &options).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_multiple_exceeds_max() {
        let base = chrono::Utc::now();
        let options = DateTimeParseOptions::for_dismissal_time(chrono_tz::Asia::Tokyo, base);

        let result = parse_datetime("1時間前, 21:00, 2日前, 3日前", &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_quest_departure_options() {
        let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);

        assert!(options
            .flags
            .contains(DateTimeParseFlags::FULL_DATETIME));
        assert!(!options.flags.contains(DateTimeParseFlags::RELATIVE_TIME));
        assert!(!options.allow_multiple);
    }

    #[test]
    fn test_schedule_start_time_options() {
        let base_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let options =
            DateTimeParseOptions::for_schedule_start_time(chrono_tz::Asia::Tokyo, base_time);

        assert!(options.flags.contains(DateTimeParseFlags::TIME_ONLY));
        assert!(options.flags.contains(DateTimeParseFlags::RELATIVE_TIME));
        assert!(!options.flags.contains(DateTimeParseFlags::FULL_DATETIME));
    }

    #[test]
    fn test_quest_departure_absolute_datetime() {
        let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);

        // 完全日時
        let result = parse_datetime("2025/11/15 21:00", &options).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ParsedDateTime::Absolute(_)));

        // 数字パターン
        let result = parse_datetime("1230", &options).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ParsedDateTime::Absolute(_)));

        // 日本語
        let result = parse_datetime("午後9時半", &options).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ParsedDateTime::Absolute(_)));
    }

    #[test]
    fn test_dismissal_time_multiple() {
        let base = chrono::Utc::now();
        let options = DateTimeParseOptions::for_dismissal_time(chrono_tz::Asia::Tokyo, base);

        let result = parse_datetime("1時間前, 21:00", &options).unwrap();
        assert_eq!(result.len(), 2);

        // 1つ目は相対時刻
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, 1);
            }
            _ => panic!("Expected Relative"),
        }

        // 2つ目は絶対時刻
        assert!(matches!(result[1], ParsedDateTime::Absolute(_)));
    }

    #[test]
    fn test_relative_time_english() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // 英語形式
        let result = parse_datetime("2hours", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, 2);
            }
            _ => panic!("Expected Relative"),
        }

        let result = parse_datetime("90m", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { minutes, .. } => {
                assert_eq!(*minutes, 90);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_quest_departure_month_correction() {
        // マルチ募集コマンドと同じ設定で「28 1000」をテスト
        let timezone = chrono_tz::Asia::Tokyo;
        let options = DateTimeParseOptions::for_quest_departure(timezone);

        let result = parse_datetime("28 1000", &options).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let now = chrono::Utc::now();
                let dt_jst = dt.with_timezone(&timezone);
                let now_jst = now.with_timezone(&timezone);

                println!("現在: {} JST", now_jst.format("%Y/%m/%d %H:%M"));
                println!("結果: {} JST", dt_jst.format("%Y/%m/%d %H:%M"));

                // 補正により未来になっているはず
                assert!(
                    dt >= &now,
                    "補正が効いていません: 現在={}, 結果={}",
                    now_jst.format("%Y/%m/%d %H:%M"),
                    dt_jst.format("%Y/%m/%d %H:%M")
                );

                // 28日であることを確認
                assert_eq!(dt_jst.day(), 28);
                // 10時であることを確認
                assert_eq!(dt_jst.hour(), 10);
            }
            _ => panic!("Expected Absolute datetime"),
        }
    }
}
