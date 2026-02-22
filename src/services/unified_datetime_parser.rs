/// 統一日時パーサー
///
/// ビットフラグベースの柔軟な日時解析システム。
/// 既存の複数のパーサー（datetime_parser, TimeParserService, DismissalTimeParserService）を統合。
use crate::services::number_normalizer::normalize_numbers;
use crate::types::Result;
use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, TimeZone, Utc};
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
    Relative { days: i32, hours: i32, minutes: i32 },

    /// NaiveTime（定期募集開始時刻など）
    Time(NaiveTime),
}

/// パース済み解散時刻（後方互換性のため）
///
/// 旧`DismissalTimeParserService`の`ParsedDismissalTime`の代替。
/// `input_value`フィールドを含む、よりリッチな情報を保持。
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

    // 絶対日時を先に試行（"20時"などの曖昧入力を時刻として優先解釈）
    if let Ok(absolute) = parse_absolute_datetime(input, options) {
        return Ok(absolute);
    }

    // 相対時刻を試行（絶対日時で解釈できない場合）
    if options.flags.contains(DateTimeParseFlags::RELATIVE_TIME)
        && let Ok(relative) = parse_relative_time(input)
    {
        return Ok(relative);
    }

    Err(format!("日時のパースに失敗しました: {input}").into())
}

/// 絶対日時のパース
fn parse_absolute_datetime(input: &str, options: &DateTimeParseOptions) -> Result<ParsedDateTime> {
    // 今日/明日/today/tomorrow/next week を先に処理
    if let Some(dt) = parse_relative_day_keyword_datetime(input, options)? {
        return Ok(ParsedDateTime::Absolute(dt));
    }

    // 既存のdatetime_parserを使用して絶対日時をパース
    let dt = crate::services::datetime_parser::parse_event_date(input, options.timezone)?;
    Ok(ParsedDateTime::Absolute(dt))
}

/// 相対日付キーワード（今日/明日/today/tomorrow/next week）を絶対日時に変換
fn parse_relative_day_keyword_datetime(
    input: &str,
    options: &DateTimeParseOptions,
) -> Result<Option<DateTime<Utc>>> {
    use lazy_static::lazy_static;
    use regex::Regex;

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    lazy_static! {
        static ref RE_TODAY: Regex = Regex::new(r"(?i)^(今日|きょう|today)(?:\s*(.+))?$")
            .expect("todayキーワードRegexパターンが無効です");
        static ref RE_TOMORROW: Regex = Regex::new(r"(?i)^(明日|あした|tomorrow)(?:\s*(.+))?$")
            .expect("tomorrowキーワードRegexパターンが無効です");
        static ref RE_NEXT_WEEK: Regex = Regex::new(r"(?i)^(来週|next\s+week)(?:\s*(.+))?$")
            .expect("next weekキーワードRegexパターンが無効です");
    }

    let (day_offset, time_part) = if let Some(caps) = RE_TODAY.captures(trimmed) {
        (0_i64, caps.get(2).map(|m| m.as_str().trim().to_string()))
    } else if let Some(caps) = RE_TOMORROW.captures(trimmed) {
        (1_i64, caps.get(2).map(|m| m.as_str().trim().to_string()))
    } else if let Some(caps) = RE_NEXT_WEEK.captures(trimmed) {
        (7_i64, caps.get(2).map(|m| m.as_str().trim().to_string()))
    } else {
        return Ok(None);
    };

    let now_tz = Utc::now().with_timezone(&options.timezone);
    let base_date = now_tz.date_naive() + Duration::days(day_offset);

    let parsed_time = if let Some(tp) = time_part {
        parse_time_component(&tp, options.timezone)?
    } else {
        options
            .default_time
            .unwrap_or_else(|| NaiveTime::from_hms_opt(21, 0, 0).expect("固定時刻は有効です"))
    };

    let naive_dt = NaiveDateTime::new(base_date, parsed_time);
    let local_dt = options
        .timezone
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;

    Ok(Some(local_dt.with_timezone(&Utc)))
}

/// 時刻要素のみを抽出してNaiveTimeに変換
fn parse_time_component(input: &str, timezone: Tz) -> Result<NaiveTime> {
    use lazy_static::lazy_static;
    use regex::Regex;

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    // 既存パーサーで解釈できる形式はそちらを優先
    if let Ok(dt) = crate::services::datetime_parser::parse_event_date(trimmed, timezone) {
        return Ok(dt.with_timezone(&timezone).time());
    }

    // 英語AM/PM形式（例: "9 PM", "9:30 PM"）
    lazy_static! {
        static ref RE_AMPM: Regex = Regex::new(r"(?i)^([1-9]|1[0-2])(?::([0-5]\d))?\s*(am|pm)$")
            .expect("AM/PM時刻Regexパターンが無効です");
    }

    if let Some(caps) = RE_AMPM.captures(trimmed) {
        let mut hour = caps[1]
            .parse::<u32>()
            .map_err(|_| format!("無効な時刻です: {trimmed}"))?;
        let minute = caps
            .get(2)
            .map(|m| m.as_str().parse::<u32>())
            .transpose()
            .map_err(|_| format!("無効な時刻です: {trimmed}"))?
            .unwrap_or(0);
        let meridiem = caps
            .get(3)
            .map(|m| m.as_str().to_ascii_lowercase())
            .unwrap_or_default();

        if meridiem == "pm" && hour != 12 {
            hour += 12;
        } else if meridiem == "am" && hour == 12 {
            hour = 0;
        }

        let parsed = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| format!("無効な時刻です: {trimmed}"))?;
        return Ok(parsed);
    }

    Err(format!("時刻のパースに失敗しました: {trimmed}").into())
}

/// HH:MM厳格モードのパース
fn parse_strict_hhmm(input: &str, _options: &DateTimeParseOptions) -> Result<ParsedDateTime> {
    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() != 2 {
        return Err(format!("無効な時刻形式です: {input}（HH:MM形式で指定してください）").into());
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
fn parse_relative_time(input: &str) -> Result<ParsedDateTime> {
    use lazy_static::lazy_static;
    use regex::Regex;

    // 数字を正規化
    let normalized = normalize_numbers(input);
    let (body, direction) = extract_relative_direction(&normalized);

    lazy_static! {
        // 複数単位混在パターン: "1日2時間10分前", "1日1時間半前", "2h30m", "1 day 2 hours 10 minutes before"
        static ref RE_MULTI_UNIT: Regex = Regex::new(
            r"(?x)
            ^
            (?:(\d+)\s*(日|days?))?\s*                        # グループ1,2: 日（オプション）
            (?:(\d+)\s*(時間|hours?|h)\s*(半)?)?\s*          # グループ3,4,5: 時間と「半」（オプション）
            (?:(\d+)\s*(分|minutes?|mins?|m))?\s*             # グループ6,7: 分（オプション）
            $
            "
        )
        .expect("複数単位相対時刻Regexパターンが無効です");

        // 単一単位パターン（後方互換性）: "2時間前", "90m", "1day"
        static ref RE_SINGLE_UNIT: Regex = Regex::new(
            r"(?x)
            ^
            (\d+)\s*                    # 数値（スペース許可）
            (日|days?|時間|hours?|h|分|minutes?|mins?|m)      # 単位
            $
            "
        )
        .expect("単一単位相対時刻Regexパターンが無効です");

        // 「X時間半」パターン: "1時間半前", "2時間半"
        static ref RE_HOUR_HALF: Regex = Regex::new(
            r"(?x)
            ^
            (\d+)\s*                    # 数値
            (時間|hours?|h)\s*          # 時間単位
            半\s*                        # 「半」
            $
            "
        )
        .expect("時間半Regexパターンが無効です");
    }

    // パターン1: 「X時間半」パターン（単独）
    if let Some(caps) = RE_HOUR_HALF.captures(body) {
        let hours = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("数値のパースに失敗しました: {}", &caps[1]))?;

        return Ok(ParsedDateTime::Relative {
            days: 0,
            hours: hours * direction,
            minutes: 30 * direction, // 「半」は30分
        });
    }

    // パターン2: 複数単位混在パターン
    if let Some(caps) = RE_MULTI_UNIT.captures(body) {
        // グループ1: 日の数値
        let days = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);

        // グループ3: 時間の数値
        let hours = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);

        // グループ5: 「半」の有無
        let has_hour_half = caps.get(5).is_some();
        let hour_half_minutes = if has_hour_half { 30 } else { 0 };

        // グループ6: 分の数値
        let minutes = caps
            .get(6)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0)
            + hour_half_minutes;

        // 少なくとも1つの単位が指定されているか確認
        if days > 0 || hours > 0 || minutes > 0 {
            return Ok(ParsedDateTime::Relative {
                days: days * direction,
                hours: hours * direction,
                minutes: minutes * direction,
            });
        }
    }

    // パターン3: 単一単位パターン（後方互換性）
    if let Some(caps) = RE_SINGLE_UNIT.captures(body) {
        let value = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("数値のパースに失敗しました: {}", &caps[1]))?;
        let unit = &caps[2];

        let (days, hours, minutes) = match unit {
            "日" | "day" | "days" => (value, 0, 0),
            "時間" | "hour" | "hours" | "h" => (0, value, 0),
            "分" | "minute" | "minutes" | "min" | "mins" | "m" => (0, 0, value),
            _ => {
                return Err(format!("不明な時間単位です: {unit}").into());
            }
        };

        return Ok(ParsedDateTime::Relative {
            days: days * direction,
            hours: hours * direction,
            minutes: minutes * direction,
        });
    }

    Err("相対時刻のパースに失敗".to_string().into())
}

/// 相対時刻の方向を抽出
///
/// 戻り値:
/// - `&str`: 方向語を除いた本体
/// - `i32`: 方向（前/ago/before=1, 後/later/after=-1）
fn extract_relative_direction(input: &str) -> (&str, i32) {
    let trimmed = input.trim();

    if let Some(rest) = trimmed.strip_suffix('前') {
        return (rest.trim_end(), 1);
    }
    if let Some(rest) = trimmed.strip_suffix('後') {
        return (rest.trim_end(), -1);
    }

    let lowered = trimmed.to_ascii_lowercase();
    for (suffix, direction) in [
        (" before", 1),
        (" ago", 1),
        (" later", -1),
        (" after", -1),
        ("before", 1),
        ("ago", 1),
        ("later", -1),
        ("after", -1),
    ] {
        if lowered.ends_with(suffix) {
            let body_len = trimmed.len().saturating_sub(suffix.len());
            return (trimmed[..body_len].trim_end(), direction);
        }
    }

    // 方向指定なしは後方互換性のため「前」扱い
    (trimmed, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

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

        assert!(options.flags.contains(DateTimeParseFlags::FULL_DATETIME));
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
    fn test_schedule_start_time_parse() {
        let base_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let options =
            DateTimeParseOptions::for_schedule_start_time(chrono_tz::Asia::Tokyo, base_time);

        // HH:MM形式 - AbsoluteまたはTimeとして返される
        let result = parse_datetime("14:30", &options).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ParsedDateTime::Time(t) => {
                assert_eq!(t.hour(), 14);
                assert_eq!(t.minute(), 30);
            }
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&chrono_tz::Asia::Tokyo);
                assert_eq!(local.hour(), 14);
                assert_eq!(local.minute(), 30);
            }
            _ => panic!("Expected Time or Absolute for HH:MM format"),
        }

        // 相対時刻
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
            _ => panic!("Expected Relative for relative time"),
        }

        // 日本語時刻「20時」は時刻として優先解釈される
        let result = parse_datetime("20時", &options).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            ParsedDateTime::Time(t) => {
                assert_eq!(t.hour(), 20);
                assert_eq!(t.minute(), 0);
            }
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&chrono_tz::Asia::Tokyo);
                assert_eq!(local.hour(), 20);
                assert_eq!(local.minute(), 0);
            }
            ParsedDateTime::Relative { .. } => panic!("20時はRelativeとして解釈されるべきではない"),
        }
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

        let result = parse_datetime("30 minutes later", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { minutes, .. } => {
                assert_eq!(*minutes, -30);
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

    #[test]
    fn test_relative_time_hour_half() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // 日本語: "1時間半前"
        let result = parse_datetime("1時間半前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 1);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }

        // 「前」なし: "2時間半"
        let result = parse_datetime("2時間半", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_relative_time_multi_unit_japanese() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // "1日2時間10分前"
        let result = parse_datetime("1日2時間10分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 10);
            }
            _ => panic!("Expected Relative"),
        }

        // "2時間30分前"
        let result = parse_datetime("2時間30分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }

        // "1日3時間前"
        let result = parse_datetime("1日3時間前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 3);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_relative_time_multi_unit_english() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // "1 day 2 hours 10 minutes before"
        let result = parse_datetime("1 day 2 hours 10 minutes before", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 10);
            }
            _ => panic!("Expected Relative"),
        }

        // "2h30m before"
        let result = parse_datetime("2h30m before", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }

        // "10 min before"
        let result = parse_datetime("10 min before", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 0);
                assert_eq!(*hours, 0);
                assert_eq!(*minutes, 10);
            }
            _ => panic!("Expected Relative"),
        }

        // "1 day 2h before"
        let result = parse_datetime("1 day 2h before", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 0);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_relative_time_backward_compatibility() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // 既存のパターンが引き続き動作することを確認

        // "2時間前"
        let result = parse_datetime("2時間前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, 2);
            }
            _ => panic!("Expected Relative"),
        }

        // "90分前"
        let result = parse_datetime("90分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { minutes, .. } => {
                assert_eq!(*minutes, 90);
            }
            _ => panic!("Expected Relative"),
        }

        // "1day"
        let result = parse_datetime("1day", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { days, .. } => {
                assert_eq!(*days, 1);
            }
            _ => panic!("Expected Relative"),
        }

        // "2時間後" は負値で表現される（基準より後）
        let result = parse_datetime("2時間後", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, -2);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_parse_relative_day_keywords_japanese() {
        let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);
        let timezone = chrono_tz::Asia::Tokyo;
        let now_jst = Utc::now().with_timezone(&timezone);

        let result = parse_datetime("今日 21:00", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(local.date_naive(), now_jst.date_naive());
                assert_eq!(local.hour(), 21);
                assert_eq!(local.minute(), 0);
            }
            _ => panic!("Expected Absolute"),
        }

        let result = parse_datetime("明日 21時半", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(1)
                );
                assert_eq!(local.hour(), 21);
                assert_eq!(local.minute(), 30);
            }
            _ => panic!("Expected Absolute"),
        }

        let result = parse_datetime("明日21時", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(1)
                );
                assert_eq!(local.hour(), 21);
                assert_eq!(local.minute(), 0);
            }
            _ => panic!("Expected Absolute"),
        }

        let result = parse_datetime("明日22時半", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(1)
                );
                assert_eq!(local.hour(), 22);
                assert_eq!(local.minute(), 30);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_parse_relative_day_keywords_english() {
        let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);
        let timezone = chrono_tz::Asia::Tokyo;
        let now_jst = Utc::now().with_timezone(&timezone);

        let result = parse_datetime("tomorrow 2200", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(1)
                );
                assert_eq!(local.hour(), 22);
                assert_eq!(local.minute(), 0);
            }
            _ => panic!("Expected Absolute"),
        }

        let result = parse_datetime("next week 9 PM", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(7)
                );
                assert_eq!(local.hour(), 21);
                assert_eq!(local.minute(), 0);
            }
            _ => panic!("Expected Absolute"),
        }

        let result = parse_datetime("tomorrow2200", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Absolute(dt) => {
                let local = dt.with_timezone(&timezone);
                assert_eq!(
                    local.date_naive(),
                    now_jst.date_naive() + chrono::Duration::days(1)
                );
                assert_eq!(local.hour(), 22);
                assert_eq!(local.minute(), 0);
            }
            _ => panic!("Expected Absolute"),
        }
    }

    #[test]
    fn test_relative_time_with_hour_half_in_multi_unit() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // "1日1時間半前"
        let result = parse_datetime("1日1時間半前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 1);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }

        // "2日3時間半前"
        let result = parse_datetime("2日3時間半前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 2);
                assert_eq!(*hours, 3);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_normalize_numbers_fullwidth() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // 全角数字: "２時間前"
        let result = parse_datetime("２時間前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, 2);
            }
            _ => panic!("Expected Relative"),
        }

        // 全角数字混在: "１日２時間３０分前"
        let result = parse_datetime("１日２時間３０分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }
    }

    #[test]
    fn test_normalize_numbers_kanji() {
        let options = DateTimeParseOptions {
            flags: DateTimeParseFlags::RELATIVE_TIME,
            timezone: chrono_tz::Asia::Tokyo,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        };

        // 漢数字: "二時間前"
        let result = parse_datetime("二時間前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { hours, .. } => {
                assert_eq!(*hours, 2);
            }
            _ => panic!("Expected Relative"),
        }

        // 漢数字「十」: "十分前" → "10分前"
        let result = parse_datetime("十分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative { minutes, .. } => {
                assert_eq!(*minutes, 10);
            }
            _ => panic!("Expected Relative"),
        }

        // 漢数字混在: "一日二時間三十分前"
        let result = parse_datetime("一日二時間三十分前", &options).unwrap();
        match &result[0] {
            ParsedDateTime::Relative {
                days,
                hours,
                minutes,
            } => {
                assert_eq!(*days, 1);
                assert_eq!(*hours, 2);
                assert_eq!(*minutes, 30);
            }
            _ => panic!("Expected Relative"),
        }
    }
}
