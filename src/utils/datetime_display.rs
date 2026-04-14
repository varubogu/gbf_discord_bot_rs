//! 日時表示用ユーティリティ
//!
//! 曜日付き日時表示と、月日からのJST基準日付解決を提供する。

use crate::services::message::MessageTextId;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc, Weekday};
use rust_i18n::t;

/// 指定した日時を曜日付きでフォーマットする
///
/// `format_template` 内の `{weekday}` はロケール別の曜日短縮名へ置換される。
pub fn format_datetime_with_weekday<Tz: TimeZone>(
    date_time: DateTime<Tz>,
    format_template: &str,
    locale: &str,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let weekday = weekday_short_label(date_time.weekday(), locale);
    let effective_template = ensure_weekday_placeholder(format_template);
    let effective_format = effective_template.replace("{weekday}", &weekday);
    date_time.format(&effective_format).to_string()
}

/// 指定した月日を、JST基準で次回到来する日付へ解決する
pub fn resolve_next_jst_date_from_month_day(month: i32, day: i32) -> Option<NaiveDate> {
    resolve_next_jst_date_from_month_day_with_now(month, day, Utc::now())
}

/// 指定した月日を、与えた現在時刻から見てJST基準の次回到来日付へ解決する
pub fn resolve_next_jst_date_from_month_day_with_now(
    month: i32,
    day: i32,
    now_utc: DateTime<Utc>,
) -> Option<NaiveDate> {
    let month_u = u32::try_from(month).ok()?;
    let day_u = u32::try_from(day).ok()?;

    let now_jst = now_utc.with_timezone(&chrono_tz::Asia::Tokyo);
    let today_jst = now_jst.date_naive();
    let year = today_jst.year();

    let candidate_this_year = NaiveDate::from_ymd_opt(year, month_u, day_u);
    let candidate_next_year = NaiveDate::from_ymd_opt(year + 1, month_u, day_u);

    match candidate_this_year {
        Some(date) if date >= today_jst => Some(date),
        Some(_) => candidate_next_year,
        None => candidate_next_year,
    }
}

/// 指定した月日（JST基準）の曜日トークン（例: `(水)`）を返す
pub fn weekday_token_for_month_day_jst(month: i32, day: i32, locale: &str) -> Option<String> {
    let date = resolve_next_jst_date_from_month_day(month, day)?;
    let weekday = weekday_short_label(date.weekday(), locale);
    Some(format!("({weekday})"))
}

/// 日時チャンネル名（日本語）を生成する（例: `4月20日_月`）
pub fn format_date_channel_name_ja(date: NaiveDate) -> String {
    let weekday = weekday_short_label(date.weekday(), "ja");
    format!("{}月{}日_{weekday}", date.month(), date.day())
}

/// 指定曜日のロケール別短縮名を返す（ja: 月, en: Mon）
pub fn weekday_short_label(weekday: Weekday, locale: &str) -> String {
    let message_id = match weekday {
        Weekday::Mon => MessageTextId::SchedulePresenterDaysMonday,
        Weekday::Tue => MessageTextId::SchedulePresenterDaysTuesday,
        Weekday::Wed => MessageTextId::SchedulePresenterDaysWednesday,
        Weekday::Thu => MessageTextId::SchedulePresenterDaysThursday,
        Weekday::Fri => MessageTextId::SchedulePresenterDaysFriday,
        Weekday::Sat => MessageTextId::SchedulePresenterDaysSaturday,
        Weekday::Sun => MessageTextId::SchedulePresenterDaysSunday,
    };

    t!(message_id.as_str(), locale = locale).to_string()
}

/// 既存フォーマットに `{weekday}` が含まれない場合の補完
///
/// - 基本は日付の `%d` 直後に ` ({weekday})` を挿入
/// - `%d` が無い場合は末尾に追加
fn ensure_weekday_placeholder(format_template: &str) -> String {
    if format_template.contains("{weekday}") {
        return format_template.to_string();
    }

    if format_template.contains("%a") || format_template.contains("%A") {
        return format_template.to_string();
    }

    if let Some(day_pos) = format_template.find("%d") {
        let insert_pos = day_pos + 2;
        let (head, tail) = format_template.split_at(insert_pos);
        return format!("{head} ({{weekday}}){tail}");
    }

    format!("{format_template} ({{weekday}})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDate, Utc};

    #[test]
    fn format_datetime_with_weekday_ja_and_en() {
        let dt = chrono_tz::Asia::Tokyo
            .with_ymd_and_hms(2026, 4, 1, 23, 59, 0)
            .single()
            .expect("valid datetime");

        let ja = format_datetime_with_weekday(dt, "%Y/%m/%d ({weekday}) %H:%M", "ja");
        let en = format_datetime_with_weekday(dt, "%Y/%m/%d ({weekday}) %H:%M", "en");

        assert_eq!(ja, "2026/04/01 (水) 23:59");
        assert_eq!(en, "2026/04/01 (Wed) 23:59");
    }

    #[test]
    fn format_datetime_with_weekday_injects_placeholder_when_missing() {
        let dt = chrono_tz::Asia::Tokyo
            .with_ymd_and_hms(2026, 4, 1, 23, 59, 0)
            .single()
            .expect("valid datetime");

        let ja = format_datetime_with_weekday(dt, "%Y/%m/%d %H:%M", "ja");
        assert_eq!(ja, "2026/04/01 (水) 23:59");
    }

    #[test]
    fn format_datetime_with_weekday_does_not_double_inject_when_percent_a_exists() {
        let dt = chrono_tz::Asia::Tokyo
            .with_ymd_and_hms(2026, 4, 1, 23, 59, 0)
            .single()
            .expect("valid datetime");

        let result = format_datetime_with_weekday(dt, "%Y/%m/%d(%a) %H:%M", "ja");
        assert_eq!(result, "2026/04/01(Wed) 23:59");
    }

    #[test]
    fn resolve_next_jst_date_handles_year_boundary() {
        let now_utc = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2026, 12, 30)
                .expect("valid date")
                .and_hms_opt(0, 0, 0)
                .expect("valid time"),
            Utc,
        );

        let date = resolve_next_jst_date_from_month_day_with_now(1, 1, now_utc)
            .expect("should resolve date");
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2027, 1, 1).expect("valid date")
        );
    }

    #[test]
    fn resolve_next_jst_date_returns_none_on_invalid_date() {
        let now_utc = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2026, 4, 1)
                .expect("valid date")
                .and_hms_opt(0, 0, 0)
                .expect("valid time"),
            Utc,
        );

        assert!(resolve_next_jst_date_from_month_day_with_now(2, 30, now_utc).is_none());
    }

    #[test]
    fn format_date_channel_name_ja_uses_underscore_before_weekday() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 20).expect("valid date");
        assert_eq!(format_date_channel_name_ja(date), "4月20日_月");
    }
}
