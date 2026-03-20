use super::{DateTimeParseFlags, DateTimeParseOptions};
use crate::services::datetime_parser::{
    DateTimeComponents, parse_event_date, parse_event_date_with_components,
};
use crate::services::number_normalizer::normalize_numbers;
use crate::types::Result;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;

/// 絶対日時候補の分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AbsoluteCategory {
    FullDateTime,
    DateTimeNoYear,
    DateOnly,
    TimeOnly,
}

impl AbsoluteCategory {
    /// カテゴリに対応する必須フラグ
    pub(super) const fn required_flag(self) -> DateTimeParseFlags {
        match self {
            Self::FullDateTime => DateTimeParseFlags::FULL_DATETIME,
            Self::DateTimeNoYear => DateTimeParseFlags::DATETIME_NO_YEAR,
            Self::DateOnly => DateTimeParseFlags::DATE_ONLY,
            Self::TimeOnly => DateTimeParseFlags::TIME_ONLY,
        }
    }
}

/// 絶対日時の解析候補
#[derive(Debug, Clone)]
pub(super) struct AbsoluteParseCandidate {
    pub(super) datetime: DateTime<Utc>,
    pub(super) category: AbsoluteCategory,
    pub(super) source_flags: DateTimeParseFlags,
}

impl AbsoluteParseCandidate {
    /// 現在の候補が指定フラグで許可されるか
    pub(super) fn is_allowed_by(&self, flags: DateTimeParseFlags) -> bool {
        if !flags.contains(self.category.required_flag()) {
            return false;
        }

        // 日本語表現が含まれる入力は、用途側で日本語許可が必要
        if self
            .source_flags
            .contains(DateTimeParseFlags::JAPANESE_DATETIME)
            && !flags.contains(DateTimeParseFlags::JAPANESE_DATETIME)
        {
            return false;
        }

        // 数字特化表現が含まれる入力は、用途側で数字パターン許可が必要
        if self
            .source_flags
            .contains(DateTimeParseFlags::NUMERIC_PATTERNS)
            && !flags.contains(DateTimeParseFlags::NUMERIC_PATTERNS)
        {
            return false;
        }

        true
    }
}

/// 絶対日時候補を解析
pub(super) fn parse_absolute_candidate(
    input: &str,
    options: &DateTimeParseOptions,
) -> Result<Option<AbsoluteParseCandidate>> {
    // 1. 今日/明日/tomorrow/next week などを先に処理
    if let Some(candidate) = parse_relative_day_keyword_datetime(input, options)? {
        return Ok(Some(candidate));
    }

    // 2. 追加フォーマット（新規要件）
    if let Some(candidate) = parse_extended_absolute_formats(input, options.timezone)? {
        return Ok(Some(candidate));
    }

    // 3. 既存パーサーへフォールバック
    if let Ok(parsed) = parse_event_date_with_components(input, options.timezone) {
        let category = infer_absolute_category(input, parsed.components);
        let source_flags = infer_source_flags(input, category);
        let corrected_datetime =
            parse_event_date(input, options.timezone).unwrap_or(parsed.datetime);
        return Ok(Some(AbsoluteParseCandidate {
            datetime: corrected_datetime,
            category,
            source_flags,
        }));
    }

    Ok(None)
}

/// 相対日付キーワード（今日/明日/today/tomorrow/next week）を絶対日時に変換
fn parse_relative_day_keyword_datetime(
    input: &str,
    options: &DateTimeParseOptions,
) -> Result<Option<AbsoluteParseCandidate>> {
    lazy_static! {
        static ref RE_TODAY: Regex = Regex::new(r"(?i)^(今日|きょう|today)(?:\s*(.+))?$")
            .expect("todayキーワードRegexパターンが無効です");
        static ref RE_TOMORROW: Regex = Regex::new(r"(?i)^(明日|あした|tomorrow)(?:\s*(.+))?$")
            .expect("tomorrowキーワードRegexパターンが無効です");
        static ref RE_NEXT_WEEK: Regex = Regex::new(r"(?i)^(来週|next\s+week)(?:\s*(.+))?$")
            .expect("next weekキーワードRegexパターンが無効です");
    }

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

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

    let local_dt = options
        .timezone
        .from_local_datetime(&NaiveDateTime::new(base_date, parsed_time))
        .single()
        .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;

    Ok(Some(AbsoluteParseCandidate {
        datetime: local_dt.with_timezone(&Utc),
        category: AbsoluteCategory::FullDateTime,
        source_flags: DateTimeParseFlags::FULL_DATETIME,
    }))
}

/// 時刻要素のみを抽出してNaiveTimeに変換
fn parse_time_component(input: &str, timezone: Tz) -> Result<NaiveTime> {
    lazy_static! {
        static ref RE_AMPM: Regex = Regex::new(r"(?i)^([1-9]|1[0-2])(?::([0-5]\d))?\s*(am|pm)$")
            .expect("AM/PM時刻Regexパターンが無効です");
    }

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    // 既存パーサーで解釈できる形式はそちらを優先
    if let Ok(dt) = parse_event_date(trimmed, timezone) {
        return Ok(dt.with_timezone(&timezone).time());
    }

    // 英語AM/PM形式（例: "9 PM", "9:30 PM"）
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

/// 追加フォーマット（新規要件）
fn parse_extended_absolute_formats(
    input: &str,
    timezone: Tz,
) -> Result<Option<AbsoluteParseCandidate>> {
    lazy_static! {
        // 例: 2026年3月16日21時 / 2026年3月16日21時30分
        static ref RE_JP_YMD_HM: Regex = Regex::new(
            r"^(\d{4})年(\d{1,2})月(\d{1,2})日\s*(\d{1,2})時(?:(\d{1,2})分)?$"
        )
        .expect("和暦風年月日時分Regexパターンが無効です");

        // 例: 202603162100
        static ref RE_YYYYMMDDHHMM: Regex = Regex::new(r"^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})$")
            .expect("YYYYMMDDHHMM Regexパターンが無効です");

        // 例: 3/16 2100
        static ref RE_MD_HHMM: Regex = Regex::new(r"^(\d{1,2})/(\d{1,2})\s+(\d{4})$")
            .expect("M/D HHMM Regexパターンが無効です");
    }

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    if let Some(caps) = RE_JP_YMD_HM.captures(trimmed) {
        let year = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("年のパースに失敗しました: {}", &caps[1]))?;
        let month = caps[2]
            .parse::<u32>()
            .map_err(|_| format!("月のパースに失敗しました: {}", &caps[2]))?;
        let day = caps[3]
            .parse::<u32>()
            .map_err(|_| format!("日のパースに失敗しました: {}", &caps[3]))?;
        let hour = caps[4]
            .parse::<u32>()
            .map_err(|_| format!("時のパースに失敗しました: {}", &caps[4]))?;
        let minute = caps
            .get(5)
            .map(|m| m.as_str().parse::<u32>())
            .transpose()
            .map_err(|_| format!("分のパースに失敗しました: {trimmed}"))?
            .unwrap_or(0);

        let dt = build_utc_datetime(year, month, day, hour, minute, timezone)?;
        return Ok(Some(AbsoluteParseCandidate {
            datetime: dt,
            category: AbsoluteCategory::FullDateTime,
            source_flags: DateTimeParseFlags::FULL_DATETIME | DateTimeParseFlags::JAPANESE_DATETIME,
        }));
    }

    if let Some(caps) = RE_YYYYMMDDHHMM.captures(trimmed) {
        let year = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("年のパースに失敗しました: {}", &caps[1]))?;
        let month = caps[2]
            .parse::<u32>()
            .map_err(|_| format!("月のパースに失敗しました: {}", &caps[2]))?;
        let day = caps[3]
            .parse::<u32>()
            .map_err(|_| format!("日のパースに失敗しました: {}", &caps[3]))?;
        let hour = caps[4]
            .parse::<u32>()
            .map_err(|_| format!("時のパースに失敗しました: {}", &caps[4]))?;
        let minute = caps[5]
            .parse::<u32>()
            .map_err(|_| format!("分のパースに失敗しました: {}", &caps[5]))?;

        let dt = build_utc_datetime(year, month, day, hour, minute, timezone)?;
        return Ok(Some(AbsoluteParseCandidate {
            datetime: dt,
            category: AbsoluteCategory::FullDateTime,
            source_flags: DateTimeParseFlags::FULL_DATETIME | DateTimeParseFlags::NUMERIC_PATTERNS,
        }));
    }

    if let Some(caps) = RE_MD_HHMM.captures(trimmed) {
        let month = caps[1]
            .parse::<u32>()
            .map_err(|_| format!("月のパースに失敗しました: {}", &caps[1]))?;
        let day = caps[2]
            .parse::<u32>()
            .map_err(|_| format!("日のパースに失敗しました: {}", &caps[2]))?;
        let hhmm = &caps[3];
        let hour = hhmm[0..2]
            .parse::<u32>()
            .map_err(|_| format!("時のパースに失敗しました: {hhmm}"))?;
        let minute = hhmm[2..4]
            .parse::<u32>()
            .map_err(|_| format!("分のパースに失敗しました: {hhmm}"))?;

        let now_tz = Utc::now().with_timezone(&timezone);
        let mut year = now_tz.year();
        let mut dt = build_utc_datetime(year, month, day, hour, minute, timezone)?;

        // 年未指定入力は未来へ補正
        if dt < Utc::now() {
            year += 1;
            dt = build_utc_datetime(year, month, day, hour, minute, timezone)?;
        }

        return Ok(Some(AbsoluteParseCandidate {
            datetime: dt,
            category: AbsoluteCategory::DateTimeNoYear,
            source_flags: DateTimeParseFlags::DATETIME_NO_YEAR
                | DateTimeParseFlags::NUMERIC_PATTERNS,
        }));
    }

    Ok(None)
}

/// ローカル日時をUTCへ変換
fn build_utc_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    timezone: Tz,
) -> Result<DateTime<Utc>> {
    let naive_date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("無効な日付です: {year:04}-{month:02}-{day:02}"))?;
    let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| format!("無効な時刻です: {hour:02}:{minute:02}"))?;
    let naive_dt = NaiveDateTime::new(naive_date, naive_time);

    let local_dt = timezone
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;

    Ok(local_dt.with_timezone(&Utc))
}

/// 解析結果カテゴリを推定
fn infer_absolute_category(input: &str, components: DateTimeComponents) -> AbsoluteCategory {
    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    if is_time_only_input(trimmed) {
        return AbsoluteCategory::TimeOnly;
    }

    if is_date_only_input(trimmed) {
        return AbsoluteCategory::DateOnly;
    }

    if is_full_datetime_input(trimmed) {
        return AbsoluteCategory::FullDateTime;
    }

    if is_datetime_no_year_input(trimmed) {
        return AbsoluteCategory::DateTimeNoYear;
    }

    // フォールバック（既存コンポーネント情報）
    if components.has_year && components.has_month && components.has_day && components.has_time {
        return AbsoluteCategory::FullDateTime;
    }

    if !components.has_year && components.has_month && components.has_day && components.has_time {
        return AbsoluteCategory::DateTimeNoYear;
    }

    if components.has_month && components.has_day && !components.has_time {
        return AbsoluteCategory::DateOnly;
    }

    AbsoluteCategory::TimeOnly
}

/// 入力スタイルを含むフラグ推定
fn infer_source_flags(input: &str, category: AbsoluteCategory) -> DateTimeParseFlags {
    let mut flags = category.required_flag();

    if looks_japanese_datetime(input) {
        flags = flags | DateTimeParseFlags::JAPANESE_DATETIME;
    }

    if looks_numeric_pattern(input) {
        flags = flags | DateTimeParseFlags::NUMERIC_PATTERNS;
    }

    flags
}

/// 時刻のみ入力か
pub(super) fn is_time_only_input(input: &str) -> bool {
    lazy_static! {
        static ref RE_COLON_TIME: Regex =
            Regex::new(r"^(?:[01]?\d|2[0-3]):[0-5]\d$").expect("コロン時刻Regexパターンが無効です");
        static ref RE_FOUR_DIGIT: Regex =
            Regex::new(r"^\d{4}$").expect("4桁時刻Regexパターンが無効です");
        static ref RE_JP_TIME: Regex = Regex::new(r"^(午前|午後)?\d{1,2}時(半|\d{1,2}分)?$")
            .expect("日本語時刻Regexパターンが無効です");
        static ref RE_AMPM: Regex = Regex::new(r"(?i)^([1-9]|1[0-2])(?::([0-5]\d))?\s*(am|pm)$")
            .expect("AM/PM時刻Regexパターンが無効です");
    }

    let normalized = normalize_numbers(input);
    let trimmed = normalized.trim();

    RE_COLON_TIME.is_match(trimmed)
        || RE_FOUR_DIGIT.is_match(trimmed)
        || RE_JP_TIME.is_match(trimmed)
        || RE_AMPM.is_match(trimmed)
}

/// 日付のみ入力か
fn is_date_only_input(input: &str) -> bool {
    lazy_static! {
        static ref RE_DATE: Regex =
            Regex::new(r"^(\d{4}[/-]\d{1,2}[/-]\d{1,2}|\d{1,2}[/-]\d{1,2}|\d{1,2}月\d{1,2}日)$")
                .expect("日付のみRegexパターンが無効です");
    }

    RE_DATE.is_match(input)
}

/// 年付き完全日時入力か
fn is_full_datetime_input(input: &str) -> bool {
    lazy_static! {
        static ref RE_FULL: Regex = Regex::new(
            r"^(\d{4}[/-]\d{1,2}[/-]\d{1,2}\s+\d{1,2}:[0-5]\d|\d{4}[/-]\d{1,2}[/-]\d{1,2}\s+\d{1,2}時\d{1,2}分|\d{4}年\d{1,2}月\d{1,2}日\s*\d{1,2}時(?:\d{1,2}分)?|\d{12})$"
        )
        .expect("完全日時Regexパターンが無効です");
    }

    RE_FULL.is_match(input)
}

/// 年なし日時入力か
fn is_datetime_no_year_input(input: &str) -> bool {
    lazy_static! {
        static ref RE_NO_YEAR: Regex = Regex::new(
            r"^(\d{1,2}[/-]\d{1,2}\s+\d{1,2}:[0-5]\d|\d{1,2}[/-]\d{1,2}\s+\d{4}|\d{4}\s+\d{4}|\d{1,2}\s+(?:\d{4}|\d{1,2}:[0-5]\d)|\d{1,2}月\d{1,2}日\d{1,2}時(?:\d{1,2}分)?|\d{8})$"
        )
        .expect("年なし日時Regexパターンが無効です");
    }

    RE_NO_YEAR.is_match(input)
}

/// 日本語表現が含まれるか
fn looks_japanese_datetime(input: &str) -> bool {
    input.contains('年')
        || input.contains('月')
        || input.contains('日')
        || input.contains('時')
        || input.contains('分')
        || input.contains("午前")
        || input.contains("午後")
        || input.contains("今日")
        || input.contains("明日")
        || input.contains("来週")
        || input.contains("きょう")
        || input.contains("あした")
}

/// 数字パターン表現か
fn looks_numeric_pattern(input: &str) -> bool {
    lazy_static! {
        static ref RE_NUMERIC: Regex = Regex::new(
            r"^(\d{4}|\d{8}|\d{12}|\d{1,2}\s+\d{4}|\d{4}\s+\d{4}|\d{1,2}/\d{1,2}\s+\d{4})$"
        )
        .expect("数字パターン判定Regexが無効です");
    }

    let normalized = normalize_numbers(input);
    RE_NUMERIC.is_match(normalized.trim())
}
