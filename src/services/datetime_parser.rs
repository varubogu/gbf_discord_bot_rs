/// 日時パースサービス
/// 様々な形式の日時文字列をDateTime<Local>に変換する
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use regex::Regex;
use crate::types::Result;

const DEFAULT_HOUR: u32 = 21; // デフォルト時刻（21時）

/// 日時文字列をDateTime<Local>に変換
///
/// # 対応フォーマット
/// - 日時: "2025/11/15 21:00", "2025-11-15 21:00"
/// - 日付のみ: "11/15", "11-15" (時刻は21時固定)
/// - 時刻のみ: "21:00", "21時" (当日、過ぎていたら翌日)
/// - 日本語: "1月2日3時4分", "午後9時半"
pub fn parse_event_date(date_str: &str) -> Result<DateTime<Local>> {
    let trimmed = date_str.trim();

    // 1. 日本語形式を試行
    if let Ok(dt) = parse_japanese_datetime(trimmed) {
        return Ok(dt);
    }

    // 2. 完全な日時形式 (yyyy/MM/dd HH:mm, yyyy-MM-dd HH:mm)
    if let Ok(dt) = parse_full_datetime(trimmed) {
        return Ok(dt);
    }

    // 3. 日付のみ (MM/dd, MM-dd, yyyy/MM/dd, yyyy-MM-dd)
    if let Ok(dt) = parse_date_only(trimmed) {
        return Ok(dt);
    }

    // 4. 時刻のみ (HH:mm, H:mm)
    if let Ok(dt) = parse_time_only(trimmed) {
        return Ok(dt);
    }

    Err(format!("日時のパースに失敗しました: {}", date_str).into())
}

/// 完全な日時形式をパース (yyyy/MM/dd HH:mm, yyyy-MM-dd HH:mm)
fn parse_full_datetime(s: &str) -> Result<DateTime<Local>> {
    // スペース区切りで日付と時刻を分離
    let patterns = vec![
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H時%M分",
        "%Y-%m-%d %H時%M分",
    ];

    for pattern in patterns {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, pattern) {
            return Ok(Local.from_local_datetime(&naive_dt).unwrap());
        }
    }

    Err("完全日時形式のパースに失敗".to_string().into())
}

/// 日付のみをパース (時刻は21時固定)
fn parse_date_only(s: &str) -> Result<DateTime<Local>> {
    let current_year = Local::now().year();

    // yyyy/MM/dd または yyyy-MM-dd 形式
    let patterns_with_year = vec!["%Y/%m/%d", "%Y-%m-%d"];
    for pattern in patterns_with_year {
        if let Ok(naive_date) = NaiveDate::parse_from_str(s, pattern) {
            let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0).unwrap();
            let naive_dt = NaiveDateTime::new(naive_date, naive_time);
            return Ok(Local.from_local_datetime(&naive_dt).unwrap());
        }
    }

    // MM/dd または MM-dd 形式（年は現在年）
    let patterns_without_year = vec!["%m/%d", "%m-%d"];
    for pattern in patterns_without_year {
        let with_year = format!("{}/{}", current_year, s);
        if let Ok(naive_date) = NaiveDate::parse_from_str(&with_year, &format!("%Y/{}", pattern)) {
            let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0).unwrap();
            let naive_dt = NaiveDateTime::new(naive_date, naive_time);
            return Ok(Local.from_local_datetime(&naive_dt).unwrap());
        }
    }

    Err("日付のみ形式のパースに失敗".to_string().into())
}

/// 時刻のみをパース (当日、過ぎていたら翌日)
fn parse_time_only(s: &str) -> Result<DateTime<Local>> {
    let now = Local::now();

    // HH:mm 形式
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H:%M") {
        return create_datetime_from_time(now, naive_time);
    }

    // H:mm 形式（1桁の時）
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%-H:%M") {
        return create_datetime_from_time(now, naive_time);
    }

    // HH時mm分 形式
    if let Ok(naive_time) = NaiveTime::parse_from_str(s, "%H時%M分") {
        return create_datetime_from_time(now, naive_time);
    }

    Err("時刻のみ形式のパースに失敗".to_string().into())
}

/// 時刻から日時を生成（過ぎていたら翌日）
fn create_datetime_from_time(now: DateTime<Local>, time: NaiveTime) -> Result<DateTime<Local>> {
    let today = now.date_naive();
    let naive_dt = NaiveDateTime::new(today, time);
    let mut dt = Local.from_local_datetime(&naive_dt).unwrap();

    // 指定時刻が現在時刻より前なら翌日にする
    if dt <= now {
        dt = dt + chrono::Duration::days(1);
    }

    Ok(dt)
}

/// 日本語形式の日時をパース
fn parse_japanese_datetime(s: &str) -> Result<DateTime<Local>> {
    // "1月2日3時4分" 形式
    let re_full = Regex::new(r"^(\d+)月(\d+)日(\d+)時(\d+)分$").unwrap();
    if let Some(caps) = re_full.captures(s) {
        let month: u32 = caps[1].parse().map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2].parse().map_err(|_| "日のパースエラー".to_string())?;
        let hour: u32 = caps[3].parse().map_err(|_| "時のパースエラー".to_string())?;
        let minute: u32 = caps[4].parse().map_err(|_| "分のパースエラー".to_string())?;

        let year = Local::now().year();
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "無効な日付".to_string())?;
        let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| "無効な時刻".to_string())?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        return Ok(Local.from_local_datetime(&naive_dt).unwrap());
    }

    // "午後9時半" 形式
    let re_time = Regex::new(r"^(午前|午後)?(\d+)時(半|(\d+)分)?$").unwrap();
    if let Some(caps) = re_time.captures(s) {
        let is_pm = caps.get(1).map_or(false, |m| m.as_str() == "午後");
        let mut hour: u32 = caps[2].parse().map_err(|_| "時のパースエラー".to_string())?;

        // 午後の場合、12時間加算（ただし12時は例外）
        if is_pm && hour != 12 {
            hour += 12;
        } else if !is_pm && hour == 12 {
            hour = 0;
        }

        let minute: u32 = if caps.get(3).map_or(false, |m| m.as_str() == "半") {
            30
        } else if let Some(m) = caps.get(4) {
            m.as_str().parse().map_err(|_| "分のパースエラー".to_string())?
        } else {
            0
        };

        let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| "無効な時刻".to_string())?;

        return create_datetime_from_time(Local::now(), naive_time);
    }

    // "1月2日" 形式（時刻は21時固定）
    let re_date = Regex::new(r"^(\d+)月(\d+)日$").unwrap();
    if let Some(caps) = re_date.captures(s) {
        let month: u32 = caps[1].parse().map_err(|_| "月のパースエラー".to_string())?;
        let day: u32 = caps[2].parse().map_err(|_| "日のパースエラー".to_string())?;

        let year = Local::now().year();
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "無効な日付".to_string())?;
        let naive_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, 0, 0).unwrap();
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        return Ok(Local.from_local_datetime(&naive_dt).unwrap());
    }

    Err("日本語形式のパースに失敗".to_string().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_full_datetime_slash() {
        let result = parse_event_date("2025/11/15 21:30").unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_full_datetime_hyphen() {
        let result = parse_event_date("2025-11-15 21:30").unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_date_only() {
        let result = parse_event_date("11/15").unwrap();
        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 21); // デフォルト時刻
    }

    #[test]
    fn test_parse_japanese_full() {
        let result = parse_event_date("1月2日3時4分").unwrap();
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 2);
        assert_eq!(result.hour(), 3);
        assert_eq!(result.minute(), 4);
    }

    #[test]
    fn test_parse_japanese_pm_half() {
        let result = parse_event_date("午後9時半").unwrap();
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 30);
    }

    #[test]
    fn test_parse_japanese_date_only() {
        let result = parse_event_date("1月2日").unwrap();
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 2);
        assert_eq!(result.hour(), 21); // デフォルト時刻
    }
}
