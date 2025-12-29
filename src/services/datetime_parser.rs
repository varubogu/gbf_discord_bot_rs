use crate::types::Result;
/// 日時パースサービス
/// 様々な形式の日時文字列をDateTime<Utc>に変換する
/// ユーザー入力はサーバー設定のタイムゾーンとして解釈し、UTCに変換する
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;

const DEFAULT_HOUR: u32 = 21; // デフォルト時刻（21時）

/// 入力文字列に含まれていた日時要素の情報
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateTimeComponents {
    pub has_year: bool,   // 年が入力に含まれていたか
    pub has_month: bool,  // 月が入力に含まれていたか
    pub has_day: bool,    // 日が入力に含まれていたか
    pub has_time: bool,   // 時刻が入力に含まれていたか
}

impl DateTimeComponents {
    /// 完全な日時（年月日時刻すべて指定）
    pub fn full_datetime() -> Self {
        Self {
            has_year: true,
            has_month: true,
            has_day: true,
            has_time: true,
        }
    }

    /// 年なし日時（月日時刻のみ）
    pub fn without_year() -> Self {
        Self {
            has_year: false,
            has_month: true,
            has_day: true,
            has_time: true,
        }
    }

    /// 日付のみ（年月日、時刻はデフォルト）
    pub fn date_only_with_year() -> Self {
        Self {
            has_year: true,
            has_month: true,
            has_day: true,
            has_time: false,
        }
    }

    /// 日付のみ（月日、時刻はデフォルト）
    pub fn date_only_without_year() -> Self {
        Self {
            has_year: false,
            has_month: true,
            has_day: true,
            has_time: false,
        }
    }

    /// 時刻のみ（日付は当日または翌日）
    pub fn time_only() -> Self {
        Self {
            has_year: false,
            has_month: false,
            has_day: false,
            has_time: true,
        }
    }

    /// 日と時刻のみ（月は現在月を使用）
    pub fn day_and_time() -> Self {
        Self {
            has_year: false,
            has_month: false,
            has_day: true,
            has_time: true,
        }
    }
}

/// パース結果（日時 + メタデータ）
#[derive(Debug, Clone)]
pub struct ParsedDateTimeResult {
    pub datetime: DateTime<Utc>,
    pub components: DateTimeComponents,
}

lazy_static! {
    /// 日本語形式の日時パターン（1月2日3時4分）
    static ref RE_JAPANESE_FULL: Regex = Regex::new(r"^(\d+)月(\d+)日(\d+)時(\d+)分$")
        .expect("日本語形式（完全）のRegexパターンが無効です");

    /// 日本語形式の時刻パターン（午後9時半）
    static ref RE_JAPANESE_TIME: Regex = Regex::new(r"^(午前|午後)?(\d+)時(半|(\d+)分)?$")
        .expect("日本語形式（時刻）のRegexパターンが無効です");

    /// 日本語形式の日付パターン（1月2日）
    static ref RE_JAPANESE_DATE: Regex = Regex::new(r"^(\d+)月(\d+)日$")
        .expect("日本語形式（日付）のRegexパターンが無効です");

    /// 4桁数字の時刻パターン（1230 → 12:30）
    static ref RE_FOUR_DIGIT_TIME: Regex = Regex::new(r"^(\d{4})$")
        .expect("4桁時刻のRegexパターンが無効です");

    /// 8桁数字の日時パターン（10111230 → 10月11日12時30分、スペース区切りも許可）
    static ref RE_EIGHT_DIGIT_DATETIME: Regex = Regex::new(r"^(\d{2})(\d{2})\s*(\d{4})$")
        .expect("8桁日時のRegexパターンが無効です");

    /// 日と時刻の組み合わせパターン（30 1230 → 30日12時30分、30 2:15 → 30日2時15分）
    static ref RE_DAY_AND_TIME: Regex = Regex::new(r"^(\d{1,2})\s+(?:(\d{4})|(\d{1,2}):(\d{2}))$")
        .expect("日+時刻のRegexパターンが無効です");
}

/// 過去日時の自動補正
///
/// 入力に含まれていなかった日時要素に基づいて、過去日時を未来に補正する:
/// - 月が未指定で過去 → 翌月に補正（優先度高）
/// - 年が未指定で過去 → 翌年に補正（優先度低）
/// - 時刻のみの場合は既にパーサー内で翌日補正済み
///
/// ★重要: 月補正を年補正より先に試行する
/// 理由: 「28 1000」のように日+時刻のパターンでは、月も年も未指定だが、
///       より小さい単位（月）で補正したほうが自然
fn correct_past_datetime(result: ParsedDateTimeResult, timezone: Tz) -> DateTime<Utc> {
    let now = Utc::now();
    let mut corrected = result.datetime;

    // 既に未来なら補正不要
    if corrected >= now {
        return corrected;
    }

    // 月が未指定で過去の場合 → 翌月に補正（優先）
    if !result.components.has_month && corrected < now {
        use chrono::Months;
        corrected = corrected.checked_add_months(Months::new(1)).unwrap_or(corrected);

        // 翌月にしたら未来になった場合はそれを返す
        if corrected >= now {
            return corrected;
        }
    }

    // 年が未指定で過去の場合 → 翌年に補正
    if !result.components.has_year && corrected < now {
        let corrected_tz = corrected.with_timezone(&timezone);
        let next_year = corrected_tz.with_year(corrected_tz.year() + 1).unwrap_or(corrected_tz);
        corrected = next_year.with_timezone(&Utc);
    }

    corrected
}

/// 日時文字列をDateTime<Utc>に変換（メタデータ付き）
///
/// ユーザー入力はサーバー設定のタイムゾーンとして解釈し、内部的にUTCに変換する
/// 入力に含まれていた日時要素の情報も一緒に返す
///
/// # 対応フォーマット
/// - 日時: "2025/11/15 21:00", "2025-11-15 21:00", "12/11 14:00", "12-11 14:00"
/// - 日付のみ: "11/15", "11-15" (時刻は21時固定)
/// - 時刻のみ: "21:00", "2:15", "21時" (当日、過ぎていたら翌日)
/// - 日本語: "1月2日3時4分", "午後9時半", "21時", "21時半", "午後6時"
/// - 数字のみ: "1230" (12時30分), "10111230" (10月11日12時30分)
/// - 日+時刻: "30 1230" (30日12時30分), "30 2:15" (30日2時15分), "15 21:00" (15日21時)
pub fn parse_event_date_with_components(date_str: &str, timezone: Tz) -> Result<ParsedDateTimeResult> {
    let trimmed = date_str.trim();

    // 1. 数字のみのパターンを試行（優先度高: 4桁、8桁、日+4桁）
    if let Ok(result) = parse_numeric_patterns_with_components(trimmed, timezone) {
        return Ok(result);
    }

    // 2. 日本語形式を試行
    if let Ok(dt) = parse_japanese_datetime(trimmed, timezone) {
        // 日本語は `without_year` として扱う（月日時刻は指定されるが年は指定されない）
        return Ok(ParsedDateTimeResult {
            datetime: dt,
            components: DateTimeComponents::without_year(),
        });
    }

    // 3. 完全な日時形式 (yyyy/MM/dd HH:mm, yyyy-MM-dd HH:mm)
    if let Ok(dt) = parse_full_datetime(trimmed, timezone) {
        return Ok(ParsedDateTimeResult {
            datetime: dt,
            components: DateTimeComponents::full_datetime(),
        });
    }

    // 4. 年なし日時形式 (MM/dd HH:mm, MM-dd HH:mm)
    if let Ok(dt) = parse_datetime_without_year(trimmed, timezone) {
        return Ok(ParsedDateTimeResult {
            datetime: dt,
            components: DateTimeComponents::without_year(),
        });
    }

    // 5. 日付のみ (MM/dd, MM-dd, yyyy/MM/dd, yyyy-MM-dd)
    if let Ok(dt) = parse_date_only(trimmed, timezone) {
        // 年が含まれているかチェック（スラッシュまたはハイフンの数で判定）
        let has_year = trimmed.matches('/').count() == 2 || trimmed.matches('-').count() == 2;
        let components = if has_year {
            DateTimeComponents::date_only_with_year()
        } else {
            DateTimeComponents::date_only_without_year()
        };
        return Ok(ParsedDateTimeResult {
            datetime: dt,
            components,
        });
    }

    // 6. 時刻のみ (HH:mm, H:mm)
    if let Ok(dt) = parse_time_only(trimmed, timezone) {
        return Ok(ParsedDateTimeResult {
            datetime: dt,
            components: DateTimeComponents::time_only(),
        });
    }

    Err(format!("日時のパースに失敗しました: {date_str}").into())
}

/// 日時文字列をDateTime<Utc>に変換（自動補正あり）
///
/// ユーザー入力はサーバー設定のタイムゾーンとして解釈し、内部的にUTCに変換する。
/// 過去日時になる場合は自動的に未来に補正される（年・月の未指定時）。
///
/// # 対応フォーマット
/// - 日時: "2025/11/15 21:00", "2025-11-15 21:00", "12/11 14:00", "12-11 14:00"
/// - 日付のみ: "11/15", "11-15" (時刻は21時固定)
/// - 時刻のみ: "21:00", "2:15", "21時" (当日、過ぎていたら翌日)
/// - 日本語: "1月2日3時4分", "午後9時半", "21時", "21時半", "午後6時"
/// - 数字のみ: "1230" (12時30分), "10111230" (10月11日12時30分)
/// - 日+時刻: "30 1230" (30日12時30分), "30 2:15" (30日2時15分), "15 21:00" (15日21時)
///
/// # 自動補正の例
/// - 今日が12/29で「28 1000」→ 翌月の1/28 10:00（月未指定で過去のため）
/// - 今日が12/29で「1/4」→ 翌年の1/4 21:00（年未指定で過去のため）
/// - 今日が12/29で「2025/1/4」→ 2025/1/4 21:00（年指定済みのため補正なし）
pub fn parse_event_date(date_str: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let result = parse_event_date_with_components(date_str, timezone)?;
    Ok(correct_past_datetime(result, timezone))
}

/// 完全な日時形式をパース (yyyy/MM/dd HH:mm, yyyy-MM-dd HH:mm)
/// サーバー設定のタイムゾーンとして解釈し、UTCに変換
fn parse_full_datetime(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let patterns = vec![
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H時%M分",
        "%Y-%m-%d %H時%M分",
    ];

    for pattern in patterns {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, pattern) {
            // サーバー設定のタイムゾーンとして解釈し、UTCに変換
            let tz_dt = timezone
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
            return Ok(tz_dt.with_timezone(&Utc));
        }
    }

    Err("完全日時形式のパースに失敗".to_string().into())
}

/// 年なし日時形式をパース (MM/dd HH:mm, MM-dd HH:mm)
/// 年は現在年を使用し、サーバー設定のタイムゾーンとして解釈し、UTCに変換
fn parse_datetime_without_year(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let now_tz = Utc::now().with_timezone(&timezone);
    let current_year = now_tz.year();

    let patterns = vec![
        "%m/%d %H:%M",
        "%m-%d %H:%M",
        "%m/%d %H時%M分",
        "%m-%d %H時%M分",
    ];

    for pattern in patterns {
        // 年を追加してパース
        let with_year = format!("{current_year}/{s}");
        let year_pattern = format!("%Y/{pattern}");

        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&with_year, &year_pattern) {
            // サーバー設定のタイムゾーンとして解釈し、UTCに変換
            let tz_dt = timezone
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
            return Ok(tz_dt.with_timezone(&Utc));
        }
    }

    Err("年なし日時形式のパースに失敗".to_string().into())
}

/// 日付のみをパース (時刻は21時固定)
fn parse_date_only(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let now_tz = Utc::now().with_timezone(&timezone);
    let current_year = now_tz.year();

    // yyyy/MM/dd または yyyy-MM-dd 形式
    let patterns_with_year = vec!["%Y/%m/%d", "%Y-%m-%d"];
    for pattern in patterns_with_year {
        if let Ok(naive_date) = NaiveDate::parse_from_str(s, pattern) {
            let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0)
                .expect("DEFAULT_HOURは常に有効な時刻です");
            let naive_dt = NaiveDateTime::new(naive_date, naive_time);
            let tz_dt = timezone
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
            return Ok(tz_dt.with_timezone(&Utc));
        }
    }

    // MM/dd または MM-dd 形式（年は現在年）
    let patterns_without_year = vec!["%m/%d", "%m-%d"];
    for pattern in patterns_without_year {
        let with_year = format!("{current_year}/{s}");
        if let Ok(naive_date) = NaiveDate::parse_from_str(&with_year, &format!("%Y/{pattern}")) {
            let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0)
                .expect("DEFAULT_HOURは常に有効な時刻です");
            let naive_dt = NaiveDateTime::new(naive_date, naive_time);
            let tz_dt = timezone
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
            return Ok(tz_dt.with_timezone(&Utc));
        }
    }

    Err("日付のみ形式のパースに失敗".to_string().into())
}

/// 時刻のみをパース (当日、過ぎていたら翌日)
fn parse_time_only(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let now_tz = Utc::now().with_timezone(&timezone);

    // HH:mm 形式
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H:%M") {
        return create_datetime_from_time(now_tz, naive_time, timezone);
    }

    // H:mm 形式（1桁の時）
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%-H:%M") {
        return create_datetime_from_time(now_tz, naive_time, timezone);
    }

    // HH時mm分 形式
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H時%M分") {
        return create_datetime_from_time(now_tz, naive_time, timezone);
    }

    Err("時刻のみ形式のパースに失敗".to_string().into())
}

/// 時刻から日時を生成（過ぎていたら翌日）
fn create_datetime_from_time<Tz2: TimeZone>(
    now_tz: DateTime<Tz2>,
    time: NaiveTime,
    timezone: Tz,
) -> Result<DateTime<Utc>> {
    let today = now_tz.date_naive();
    let naive_dt = NaiveDateTime::new(today, time);
    let mut dt_tz = timezone
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;

    // 指定時刻が現在時刻より前なら翌日にする
    if dt_tz <= now_tz.with_timezone(&timezone) {
        dt_tz += chrono::Duration::days(1);
    }

    Ok(dt_tz.with_timezone(&Utc))
}

/// 日本語形式の日時をパース（サーバー設定のタイムゾーンとして解釈し、UTCに変換）
fn parse_japanese_datetime(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let now_tz = Utc::now().with_timezone(&timezone);

    // "1月2日3時4分" 形式
    if let Some(caps) = RE_JAPANESE_FULL.captures(s) {
        let month: u32 = caps[1]
            .parse()
            .map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2]
            .parse()
            .map_err(|_| "日のパースエラー".to_string())?;
        let hour: u32 = caps[3]
            .parse()
            .map_err(|_| "時のパースエラー".to_string())?;
        let minute: u32 = caps[4]
            .parse()
            .map_err(|_| "分のパースエラー".to_string())?;

        let year = now_tz.year();
        let naive_date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "無効な日付".to_string())?;
        let naive_time =
            NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "無効な時刻".to_string())?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let tz_dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
        return Ok(tz_dt.with_timezone(&Utc));
    }

    // "午後9時半" 形式
    if let Some(caps) = RE_JAPANESE_TIME.captures(s) {
        let is_pm = caps.get(1).is_some_and(|m| m.as_str() == "午後");
        let mut hour: u32 = caps[2]
            .parse()
            .map_err(|_| "時のパースエラー".to_string())?;

        if is_pm && hour != 12 {
            hour += 12;
        } else if !is_pm && hour == 12 {
            hour = 0;
        }

        let minute: u32 = if caps.get(3).is_some_and(|m| m.as_str() == "半") {
            30
        } else if let Some(m) = caps.get(4) {
            m.as_str()
                .parse()
                .map_err(|_| "分のパースエラー".to_string())?
        } else {
            0
        };

        let naive_time =
            NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "無効な時刻".to_string())?;

        return create_datetime_from_time(now_tz, naive_time, timezone);
    }

    // "1月2日" 形式（時刻は21時固定）
    if let Some(caps) = RE_JAPANESE_DATE.captures(s) {
        let month: u32 = caps[1]
            .parse()
            .map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2]
            .parse()
            .map_err(|_| "日のパースエラー".to_string())?;

        let year = now_tz.year();
        let naive_date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "無効な日付".to_string())?;
        let naive_time =
            NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0).expect("DEFAULT_HOURは常に有効な時刻です");
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let tz_dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
        return Ok(tz_dt.with_timezone(&Utc));
    }

    Err("日本語形式のパースに失敗".to_string().into())
}

/// 数字のみのパターンをパース
/// - 4桁: "1230" → 12時30分（当日、過ぎていたら翌日）
/// - 8桁: "10111230" → 10月11日12時30分
/// - 日+時刻: "30 1230" → 30日12時30分、"30 2:15" → 30日2時15分
/// 数字パターンのパース（メタデータ付き）
fn parse_numeric_patterns_with_components(s: &str, timezone: Tz) -> Result<ParsedDateTimeResult> {
    let now_tz = Utc::now().with_timezone(&timezone);

    // 8桁日時パターン: "10111230" または "1011 1230" → 10月11日12時30分
    if let Some(caps) = RE_EIGHT_DIGIT_DATETIME.captures(s) {
        let month: u32 = caps[1]
            .parse()
            .map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2]
            .parse()
            .map_err(|_| "日のパースエラー".to_string())?;
        let time_str = &caps[3];

        let hour: u32 = time_str[0..2]
            .parse()
            .map_err(|_| "時のパースエラー".to_string())?;
        let minute: u32 = time_str[2..4]
            .parse()
            .map_err(|_| "分のパースエラー".to_string())?;

        let year = now_tz.year();
        let naive_date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "無効な日付".to_string())?;
        let naive_time =
            NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "無効な時刻".to_string())?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let tz_dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
        return Ok(ParsedDateTimeResult {
            datetime: tz_dt.with_timezone(&Utc),
            components: DateTimeComponents::without_year(), // 月日時刻あり、年なし
        });
    }

    // 日+時刻パターン: "30 1230" → 30日12時30分、"30 2:15" → 30日2時15分
    if let Some(caps) = RE_DAY_AND_TIME.captures(s) {
        let day: u32 = caps[1]
            .parse()
            .map_err(|_| "日のパースエラー".to_string())?;

        // 4桁数字 or コロン区切り
        let (hour, minute) = if let Some(four_digit) = caps.get(2) {
            // 4桁: "1230" → 12時30分
            let time_str = four_digit.as_str();
            let hour: u32 = time_str[0..2]
                .parse()
                .map_err(|_| "時のパースエラー".to_string())?;
            let minute: u32 = time_str[2..4]
                .parse()
                .map_err(|_| "分のパースエラー".to_string())?;
            (hour, minute)
        } else {
            // コロン区切り: "2:15" → 2時15分
            let hour: u32 = caps[3]
                .parse()
                .map_err(|_| "時のパースエラー".to_string())?;
            let minute: u32 = caps[4]
                .parse()
                .map_err(|_| "分のパースエラー".to_string())?;
            (hour, minute)
        };

        // 現在の月を使用
        let year = now_tz.year();
        let month = now_tz.month();
        let naive_date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "無効な日付".to_string())?;
        let naive_time =
            NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "無効な時刻".to_string())?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let tz_dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
        return Ok(ParsedDateTimeResult {
            datetime: tz_dt.with_timezone(&Utc),
            components: DateTimeComponents::day_and_time(), // 日時刻あり、年月なし（★これが「28 1000」に適用される）
        });
    }

    // 4桁時刻パターン: "1230" → 12時30分（当日、過ぎていたら翌日）
    if let Some(caps) = RE_FOUR_DIGIT_TIME.captures(s) {
        let time_str = &caps[1];
        let hour: u32 = time_str[0..2]
            .parse()
            .map_err(|_| "時のパースエラー".to_string())?;
        let minute: u32 = time_str[2..4]
            .parse()
            .map_err(|_| "分のパースエラー".to_string())?;

        let naive_time =
            NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "無効な時刻".to_string())?;

        let datetime = create_datetime_from_time(now_tz, naive_time, timezone)?;
        return Ok(ParsedDateTimeResult {
            datetime,
            components: DateTimeComponents::time_only(), // 時刻のみ
        });
    }

    Err("数字パターンのパースに失敗".to_string().into())
}

/// 数字パターンのパース（後方互換性のため残す）
fn parse_numeric_patterns(s: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    Ok(parse_numeric_patterns_with_components(s, timezone)?.datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_full_datetime_slash_jst() {
        // 2025/11/15 21:30 JST = 2025/11/15 12:30 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("2025/11/15 21:30", timezone).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 12); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_full_datetime_hyphen_jst() {
        // 2025-11-15 21:30 JST = 2025-11-15 12:30 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("2025-11-15 21:30", timezone).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 12); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_japanese_full_jst() {
        // 1月2日3時4分 JST = 1月1日18時4分 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("1月2日3時4分", timezone).unwrap();
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 1); // UTC（日をまたぐ）
        assert_eq!(result.hour(), 18); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 4);
    }

    #[test]
    fn test_parse_full_datetime_slash_est() {
        // 2025/11/15 21:30 EST = 2025/11/16 02:30 UTC
        let timezone = chrono_tz::America::New_York;
        let result = parse_event_date("2025/11/15 21:30", timezone).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 16); // UTC（日をまたぐ）
        assert_eq!(result.hour(), 2); // UTC（EST + 5時間）
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_datetime_without_year_slash_jst() {
        // 12/11 14:00 JST（年未指定で過去なら翌年に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("12/11 14:00", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.month(), 12);
        assert_eq!(result_jst.day(), 11);
        assert_eq!(result_jst.hour(), 14);
        assert_eq!(result_jst.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_without_year_hyphen_jst() {
        // 12-11 14:00 JST（年未指定で過去なら翌年に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("12-11 14:00", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.month(), 12);
        assert_eq!(result_jst.day(), 11);
        assert_eq!(result_jst.hour(), 14);
        assert_eq!(result_jst.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_without_year_japanese_style_jst() {
        // 1/2 3時4分 JST（年未指定で過去なら翌年に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("1/2 3時4分", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.month(), 1);
        assert_eq!(result_jst.day(), 2);
        assert_eq!(result_jst.hour(), 3);
        assert_eq!(result_jst.minute(), 4);
    }

    #[test]
    fn test_parse_four_digit_time_jst() {
        // 4桁時刻: "1230" → 12時30分 JST = 3時30分 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("1230", timezone).unwrap();
        // 時刻のみの場合は当日または翌日
        assert_eq!(result.hour(), 3); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_eight_digit_datetime_jst() {
        // 8桁日時: "10111230" → 10月11日12時30分 JST（年未指定で過去なら翌年に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("10111230", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.month(), 10);
        assert_eq!(result_jst.day(), 11);
        assert_eq!(result_jst.hour(), 12);
        assert_eq!(result_jst.minute(), 30);
    }

    #[test]
    fn test_parse_eight_digit_datetime_with_space_jst() {
        // 8桁日時（スペース区切り）: "1011 1230" → 10月11日12時30分 JST（年未指定で過去なら翌年に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("1011 1230", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.month(), 10);
        assert_eq!(result_jst.day(), 11);
        assert_eq!(result_jst.hour(), 12);
        assert_eq!(result_jst.minute(), 30);
    }

    #[test]
    fn test_parse_day_and_four_digit_time_jst() {
        // 日+4桁時刻: "30 1230" → 30日12時30分 JST（月未指定で過去なら翌月に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("30 1230", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.day(), 30);
        assert_eq!(result_jst.hour(), 12);
        assert_eq!(result_jst.minute(), 30);
    }

    #[test]
    fn test_parse_day_and_colon_time_jst() {
        // 日+コロン区切り時刻: "30 21:00" → 30日21時0分 JST（月未指定で過去なら翌月に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("30 21:00", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.day(), 30);
        assert_eq!(result_jst.hour(), 21);
        assert_eq!(result_jst.minute(), 0);
    }

    #[test]
    fn test_parse_day_and_single_digit_hour_jst() {
        // 日+コロン区切り時刻（1桁時）: "15 2:15" → 15日2時15分 JST（月未指定で過去なら翌月に補正される）
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("15 2:15", timezone).unwrap();
        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);

        // 補正により未来になっている
        assert!(result >= now, "結果が過去になっています");
        assert_eq!(result_jst.day(), 15);
        assert_eq!(result_jst.hour(), 2);
        assert_eq!(result_jst.minute(), 15);
    }

    #[test]
    fn test_parse_japanese_time_with_half_jst() {
        // 日本語時刻（半）: "21時半" → 21時30分 JST = 12時30分 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("21時半", timezone).unwrap();
        assert_eq!(result.hour(), 12); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_japanese_time_pm_jst() {
        // 日本語時刻（午後）: "午後6時" → 18時 JST = 9時 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("午後6時", timezone).unwrap();
        assert_eq!(result.hour(), 9); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn test_parse_japanese_time_only_jst() {
        // 日本語時刻（時のみ）: "21時" → 21時 JST = 12時 UTC
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("21時", timezone).unwrap();
        assert_eq!(result.hour(), 12); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn test_month_correction_28_1000() {
        // 今日が12/29で「28 1000」を入力した場合、月補正が効いて翌月の28日になる
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("28 1000", timezone).unwrap();

        let now = Utc::now();
        let result_jst = result.with_timezone(&timezone);
        let now_jst = now.with_timezone(&timezone);

        println!("現在: {} JST", now_jst.format("%Y/%m/%d %H:%M"));
        println!("結果: {} JST", result_jst.format("%Y/%m/%d %H:%M"));
        println!("補正済み: {}", result >= now);
        println!("年: {}, 月: {}, 日: {}, 時: {}",
                 result_jst.year(), result_jst.month(), result_jst.day(), result_jst.hour());

        // 補正により未来になっているはず
        assert!(result >= now, "補正が効いていません");

        // 28日であることを確認
        assert_eq!(result_jst.day(), 28);
        // 10時であることを確認
        assert_eq!(result_jst.hour(), 10);
    }

    #[test]
    fn test_components_28_1000() {
        // 「28 1000」のコンポーネントフラグを確認
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date_with_components("28 1000", timezone).unwrap();

        println!("has_year: {}", result.components.has_year);
        println!("has_month: {}", result.components.has_month);
        println!("has_day: {}", result.components.has_day);
        println!("has_time: {}", result.components.has_time);

        // 「28 1000」は日と時刻のみのパターン
        assert_eq!(result.components.has_year, false, "年フラグが間違っています");
        assert_eq!(result.components.has_month, false, "月フラグが間違っています（★月補正のキー）");
        assert_eq!(result.components.has_day, true, "日フラグが間違っています");
        assert_eq!(result.components.has_time, true, "時刻フラグが間違っています");
    }
}
