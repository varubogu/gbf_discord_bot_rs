/// 日時パースサービス
/// 様々な形式の日時文字列をDateTime<Utc>に変換する
/// ユーザー入力はサーバー設定のタイムゾーンとして解釈し、UTCに変換する
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;
use crate::types::Result;

const DEFAULT_HOUR: u32 = 21; // デフォルト時刻（21時）

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
}

/// 日時文字列をDateTime<Utc>に変換
///
/// ユーザー入力はサーバー設定のタイムゾーンとして解釈し、内部的にUTCに変換する
///
/// # 対応フォーマット
/// - 日時: "2025/11/15 21:00", "2025-11-15 21:00", "12/11 14:00", "12-11 14:00"（過ぎていたら翌年）
/// - 日付のみ: "11/15", "11-15" (時刻は21時固定)
/// - 時刻のみ: "21:00", "21時" (当日、過ぎていたら翌日)
/// - 日本語: "1月2日3時4分", "午後9時半"
pub fn parse_event_date(date_str: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let trimmed = date_str.trim();

    // 1. 日本語形式を試行
    if let Ok(dt) = parse_japanese_datetime(trimmed, timezone) {
        return Ok(dt);
    }

    // 2. 完全な日時形式 (yyyy/MM/dd HH:mm, yyyy-MM-dd HH:mm)
    if let Ok(dt) = parse_full_datetime(trimmed, timezone) {
        return Ok(dt);
    }

    // 3. 年なし日時形式 (MM/dd HH:mm, MM-dd HH:mm)
    if let Ok(dt) = parse_datetime_without_year(trimmed, timezone) {
        return Ok(dt);
    }

    // 4. 日付のみ (MM/dd, MM-dd, yyyy/MM/dd, yyyy-MM-dd)
    if let Ok(dt) = parse_date_only(trimmed, timezone) {
        return Ok(dt);
    }

    // 5. 時刻のみ (HH:mm, H:mm)
    if let Ok(dt) = parse_time_only(trimmed, timezone) {
        return Ok(dt);
    }

    Err(format!("日時のパースに失敗しました: {date_str}").into())
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
        let month: u32 = caps[1].parse().map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2].parse().map_err(|_| "日のパースエラー".to_string())?;
        let hour: u32 = caps[3].parse().map_err(|_| "時のパースエラー".to_string())?;
        let minute: u32 = caps[4].parse().map_err(|_| "分のパースエラー".to_string())?;

        let year = now_tz.year();
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "無効な日付".to_string())?;
        let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| "無効な時刻".to_string())?;
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
        let mut hour: u32 = caps[2].parse().map_err(|_| "時のパースエラー".to_string())?;

        if is_pm && hour != 12 {
            hour += 12;
        } else if !is_pm && hour == 12 {
            hour = 0;
        }

        let minute: u32 = if caps.get(3).is_some_and(|m| m.as_str() == "半") {
            30
        } else if let Some(m) = caps.get(4) {
            m.as_str().parse().map_err(|_| "分のパースエラー".to_string())?
        } else {
            0
        };

        let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| "無効な時刻".to_string())?;

        return create_datetime_from_time(now_tz, naive_time, timezone);
    }

    // "1月2日" 形式（時刻は21時固定）
    if let Some(caps) = RE_JAPANESE_DATE.captures(s) {
        let month: u32 = caps[1].parse().map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2].parse().map_err(|_| "日のパースエラー".to_string())?;

        let year = now_tz.year();
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "無効な日付".to_string())?;
        let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0)
            .expect("DEFAULT_HOURは常に有効な時刻です");
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let tz_dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;
        return Ok(tz_dt.with_timezone(&Utc));
    }

    Err("日本語形式のパースに失敗".to_string().into())
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
        // 12/11 14:00 JST = 12/11 05:00 UTC (現在年を使用)
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("12/11 14:00", timezone).unwrap();
        let now = Utc::now();
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 11);
        assert_eq!(result.hour(), 5); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_without_year_hyphen_jst() {
        // 12-11 14:00 JST = 12-11 05:00 UTC (現在年を使用)
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("12-11 14:00", timezone).unwrap();
        let now = Utc::now();
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 11);
        assert_eq!(result.hour(), 5); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_without_year_japanese_style_jst() {
        // 1/2 3時4分 JST = 1/1 18時4分 UTC (現在年を使用)
        let timezone = chrono_tz::Asia::Tokyo;
        let result = parse_event_date("1/2 3時4分", timezone).unwrap();
        let now = Utc::now();
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 1); // UTC（日をまたぐ）
        assert_eq!(result.hour(), 18); // UTC（JST - 9時間）
        assert_eq!(result.minute(), 4);
    }
}
