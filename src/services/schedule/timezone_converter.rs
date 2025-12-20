use crate::types::Result;
/// タイムゾーン変換ユーティリティ
/// 定期スケジュールのローカル時刻⇔UTC変換を行う
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use sea_orm::prelude::TimeTime;

/// ローカル曜日・時刻をUTC曜日・時刻に変換
///
/// # 例
/// ```
/// // 火曜 22:00 JST → 火曜 13:00 UTC
/// // 火曜 01:00 JST → 月曜 16:00 UTC (曜日が変わる)
/// ```
pub fn convert_local_days_and_time_to_utc(
    local_days: &[i32],
    local_time: NaiveTime,
    timezone: Tz,
) -> Result<(Vec<i32>, TimeTime)> {
    // 基準日を使用（2025-01-06 = 月曜日）
    let base_date = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap();

    let mut utc_days_set = std::collections::HashSet::new();

    // 各曜日について、UTC曜日を計算
    for &local_day in local_days {
        if local_day == 0 {
            // 「毎日」の場合はUTCでも毎日
            utc_days_set.insert(0);
            continue;
        }

        // ローカル曜日の基準日を計算（1=月、2=火、...、7=日）
        let days_from_monday = (local_day - 1) as i64;
        let local_ref_date = base_date + Duration::days(days_from_monday);

        // ローカル日時を作成
        let local_datetime = local_ref_date.and_time(local_time);

        // タイムゾーンを適用してUTCに変換
        let local_tz = timezone
            .from_local_datetime(&local_datetime)
            .single()
            .ok_or_else(|| crate::types::AppError::Business {
                message: "曖昧な時刻またはサマータイム切り替え時刻です".to_string(),
            })?;

        let utc_datetime = local_tz.with_timezone(&Utc);

        // UTC曜日を取得
        let utc_weekday = utc_datetime.weekday();
        let utc_day = weekday_to_number(utc_weekday);

        utc_days_set.insert(utc_day);
    }

    let mut utc_days: Vec<i32> = utc_days_set.into_iter().collect();
    utc_days.sort();

    // UTC時刻を計算（最初の曜日を使用して計算）
    let first_local_day = if local_days.contains(&0) {
        1 // 毎日の場合は月曜日で計算
    } else {
        local_days[0]
    };

    let days_from_monday = (first_local_day - 1) as i64;
    let local_ref_date = base_date + Duration::days(days_from_monday);
    let local_datetime = local_ref_date.and_time(local_time);

    let local_tz = timezone
        .from_local_datetime(&local_datetime)
        .single()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "曖昧な時刻またはサマータイム切り替え時刻です".to_string(),
        })?;

    let utc_datetime = local_tz.with_timezone(&Utc);
    let utc_time = naive_time_to_time_time(utc_datetime.time());

    Ok((utc_days, utc_time))
}

/// UTC曜日・時刻をローカル曜日・時刻に変換
///
/// # 例
/// ```
/// // 火曜 13:00 UTC → 火曜 22:00 JST
/// // 月曜 16:00 UTC → 火曜 01:00 JST (曜日が変わる)
/// ```
pub fn convert_utc_days_and_time_to_local(
    utc_days: &[i32],
    utc_time: TimeTime,
    timezone: Tz,
) -> Result<(Vec<i32>, NaiveTime)> {
    // 基準日を使用（2025-01-06 = 月曜日）
    let base_date = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap();

    let mut local_days_set = std::collections::HashSet::new();

    // 各UTC曜日について、ローカル曜日を計算
    for &utc_day in utc_days {
        if utc_day == 0 {
            // 「毎日」の場合はローカルでも毎日
            local_days_set.insert(0);
            continue;
        }

        // UTC曜日の基準日を計算
        let days_from_monday = (utc_day - 1) as i64;
        let utc_ref_date = base_date + Duration::days(days_from_monday);

        // UTC日時を作成
        let utc_naive_time = time_time_to_naive_time(utc_time);
        let utc_datetime = utc_ref_date.and_time(utc_naive_time).and_utc();

        // ローカルタイムゾーンに変換
        let local_datetime = utc_datetime.with_timezone(&timezone);

        // ローカル曜日を取得
        let local_weekday = local_datetime.weekday();
        let local_day = weekday_to_number(local_weekday);

        local_days_set.insert(local_day);
    }

    let mut local_days: Vec<i32> = local_days_set.into_iter().collect();
    local_days.sort();

    // ローカル時刻を計算（最初のUTC曜日を使用）
    let first_utc_day = if utc_days.contains(&0) {
        1 // 毎日の場合は月曜日で計算
    } else {
        utc_days[0]
    };

    let days_from_monday = (first_utc_day - 1) as i64;
    let utc_ref_date = base_date + Duration::days(days_from_monday);
    let utc_naive_time = time_time_to_naive_time(utc_time);
    let utc_datetime = utc_ref_date.and_time(utc_naive_time).and_utc();

    let local_datetime = utc_datetime.with_timezone(&timezone);
    let local_time = local_datetime.time();

    Ok((local_days, local_time))
}

/// chrono::WeekdayをDB用の数値に変換
/// 1=月、2=火、3=水、4=木、5=金、6=土、7=日
fn weekday_to_number(weekday: Weekday) -> i32 {
    match weekday {
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
        Weekday::Sun => 7,
    }
}

/// TimeTime（SeaORM型）をNaiveTime（chrono型）に変換
fn time_time_to_naive_time(time: TimeTime) -> NaiveTime {
    NaiveTime::from_hms_opt(
        time.hour() as u32,
        time.minute() as u32,
        time.second() as u32,
    )
    .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
}

/// NaiveTimeをTimeTimeに変換
fn naive_time_to_time_time(time: NaiveTime) -> TimeTime {
    TimeTime::from_hms(time.hour() as u8, time.minute() as u8, time.second() as u8)
        .unwrap_or_else(|_| TimeTime::from_hms(0, 0, 0).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Asia::Tokyo;

    #[test]
    fn test_convert_local_to_utc_same_day() {
        // 火曜 22:00 JST → 火曜 13:00 UTC
        let local_days = vec![2]; // 火曜
        let local_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let (utc_days, utc_time) =
            convert_local_days_and_time_to_utc(&local_days, local_time, Tokyo).unwrap();

        assert_eq!(utc_days, vec![2]); // 火曜
        assert_eq!(utc_time.hour(), 13);
        assert_eq!(utc_time.minute(), 0);
    }

    #[test]
    fn test_convert_local_to_utc_day_change() {
        // 火曜 01:00 JST → 月曜 16:00 UTC
        let local_days = vec![2]; // 火曜
        let local_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let (utc_days, utc_time) =
            convert_local_days_and_time_to_utc(&local_days, local_time, Tokyo).unwrap();

        assert_eq!(utc_days, vec![1]); // 月曜
        assert_eq!(utc_time.hour(), 16);
        assert_eq!(utc_time.minute(), 0);
    }

    #[test]
    fn test_convert_utc_to_local_same_day() {
        // 火曜 13:00 UTC → 火曜 22:00 JST
        let utc_days = vec![2]; // 火曜
        let utc_time = TimeTime::from_hms(13, 0, 0).unwrap();

        let (local_days, local_time) =
            convert_utc_days_and_time_to_local(&utc_days, utc_time, Tokyo).unwrap();

        assert_eq!(local_days, vec![2]); // 火曜
        assert_eq!(local_time.hour(), 22);
        assert_eq!(local_time.minute(), 0);
    }

    #[test]
    fn test_convert_utc_to_local_day_change() {
        // 月曜 16:00 UTC → 火曜 01:00 JST
        let utc_days = vec![1]; // 月曜
        let utc_time = TimeTime::from_hms(16, 0, 0).unwrap();

        let (local_days, local_time) =
            convert_utc_days_and_time_to_local(&utc_days, utc_time, Tokyo).unwrap();

        assert_eq!(local_days, vec![2]); // 火曜
        assert_eq!(local_time.hour(), 1);
        assert_eq!(local_time.minute(), 0);
    }

    #[test]
    fn test_convert_everyday() {
        // 毎日 22:00 JST → 毎日 13:00 UTC
        let local_days = vec![0]; // 毎日
        let local_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let (utc_days, utc_time) =
            convert_local_days_and_time_to_utc(&local_days, local_time, Tokyo).unwrap();

        assert_eq!(utc_days, vec![0]); // 毎日
        assert_eq!(utc_time.hour(), 13);
    }
}
