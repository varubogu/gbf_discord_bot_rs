use super::*;
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};

#[test]
fn test_flags_union_and_contains() {
    let flags = DateTimeParseFlags::TIME_ONLY | DateTimeParseFlags::DATE_ONLY;
    assert!(flags.contains(DateTimeParseFlags::TIME_ONLY));
    assert!(flags.contains(DateTimeParseFlags::DATE_ONLY));
    assert!(!flags.contains(DateTimeParseFlags::FULL_DATETIME));
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
fn test_parse_relative_time_backward_compatibility() {
    let options = DateTimeParseOptions {
        flags: DateTimeParseFlags::RELATIVE_TIME,
        timezone: chrono_tz::Asia::Tokyo,
        relative_base: None,
        default_time: None,
        allow_multiple: false,
        max_count: 1,
    };

    let result = parse_datetime("2時間前", &options).unwrap();
    match &result[0] {
        ParsedDateTime::Relative { hours, .. } => assert_eq!(*hours, 2),
        _ => panic!("Expected Relative"),
    }
}

#[test]
fn test_schedule_start_time_returns_time_for_clock_input() {
    let base_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
    let options = DateTimeParseOptions::for_schedule_start_time(chrono_tz::Asia::Tokyo, base_time);

    let result = parse_datetime("19:00", &options).unwrap();
    match &result[0] {
        ParsedDateTime::Time(t) => {
            assert_eq!(t.hour(), 19);
            assert_eq!(t.minute(), 0);
        }
        _ => panic!("Expected Time"),
    }

    let result = parse_datetime("午後9時半", &options).unwrap();
    match &result[0] {
        ParsedDateTime::Time(t) => {
            assert_eq!(t.hour(), 21);
            assert_eq!(t.minute(), 30);
        }
        _ => panic!("Expected Time"),
    }
}

#[test]
fn test_time_only_is_rebased_for_dismissal() {
    let timezone = chrono_tz::Asia::Tokyo;
    let base_local = timezone
        .with_ymd_and_hms(2026, 3, 20, 21, 0, 0)
        .single()
        .unwrap();
    let base = base_local.with_timezone(&Utc);

    let options = DateTimeParseOptions::for_dismissal_time(timezone, base);
    let result = parse_datetime("21:00", &options).unwrap();

    match &result[0] {
        ParsedDateTime::Absolute(dt) => {
            let local = dt.with_timezone(&timezone);
            assert_eq!(
                local.date_naive(),
                NaiveDate::from_ymd_opt(2026, 3, 19).unwrap()
            );
            assert_eq!(local.hour(), 21);
            assert_eq!(local.minute(), 0);
        }
        _ => panic!("Expected Absolute"),
    }
}

#[test]
fn test_dismissal_rejects_after_direction() {
    let timezone = chrono_tz::Asia::Tokyo;
    let base = timezone
        .with_ymd_and_hms(2026, 3, 20, 21, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let options = DateTimeParseOptions::for_dismissal_time(timezone, base);

    assert!(parse_datetime("2時間後", &options).is_err());
    assert!(parse_datetime("2 hours later", &options).is_err());
}

#[test]
fn test_dismissal_respects_max_days_default() {
    let timezone = chrono_tz::Asia::Tokyo;
    let base = timezone
        .with_ymd_and_hms(2026, 3, 20, 21, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let options = DateTimeParseOptions::for_dismissal_time(timezone, base);

    let max_days = std::env::var("DISMISSAL_MAX_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(7);
    let over = max_days + 1;

    // 許容日数超過はエラー
    assert!(parse_datetime(&format!("{over}日前"), &options).is_err());

    // 境界値は許可
    assert!(parse_datetime(&format!("{max_days}日前"), &options).is_ok());
}

#[test]
fn test_relative_day_keywords_still_supported() {
    let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);

    assert!(parse_datetime("今日 21:00", &options).is_ok());
    assert!(parse_datetime("明日21時", &options).is_ok());
    assert!(parse_datetime("tomorrow 2200", &options).is_ok());
    assert!(parse_datetime("next week 9 PM", &options).is_ok());
}

#[test]
fn test_new_format_japanese_with_year_hour() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let result = parse_datetime("2026年3月16日21時", &options).unwrap();
    match &result[0] {
        ParsedDateTime::Absolute(dt) => {
            let local = dt.with_timezone(&timezone);
            assert_eq!(local.year(), 2026);
            assert_eq!(local.month(), 3);
            assert_eq!(local.day(), 16);
            assert_eq!(local.hour(), 21);
            assert_eq!(local.minute(), 0);
        }
        _ => panic!("Expected Absolute"),
    }
}

#[test]
fn test_new_format_yyyymmddhhmm() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let result = parse_datetime("202603162100", &options).unwrap();
    match &result[0] {
        ParsedDateTime::Absolute(dt) => {
            let local = dt.with_timezone(&timezone);
            assert_eq!(local.year(), 2026);
            assert_eq!(local.month(), 3);
            assert_eq!(local.day(), 16);
            assert_eq!(local.hour(), 21);
            assert_eq!(local.minute(), 0);
        }
        _ => panic!("Expected Absolute"),
    }
}

#[test]
fn test_new_format_md_hhmm() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let result = parse_datetime("3/16 2100", &options).unwrap();
    assert!(matches!(result[0], ParsedDateTime::Absolute(_)));
}

#[test]
fn test_new_format_mmdd_hhmm() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let result = parse_datetime("0316 2100", &options).unwrap();
    assert!(matches!(result[0], ParsedDateTime::Absolute(_)));
}

#[test]
fn test_new_format_mmddhhmm() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let result = parse_datetime("03162100", &options).unwrap();
    assert!(matches!(result[0], ParsedDateTime::Absolute(_)));
}

#[test]
fn test_new_format_equivalence_dynamic_year() {
    let timezone = chrono_tz::Asia::Tokyo;
    let options = DateTimeParseOptions::for_quest_departure(timezone);

    let md_result = parse_datetime("3/16 2100", &options).unwrap();
    let md_dt = match &md_result[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => panic!("Expected Absolute"),
    };
    let resolved_year = md_dt.with_timezone(&timezone).year();

    let jp_input = format!("{}年3月16日21時", resolved_year);
    let ymd_input = format!("{}03162100", resolved_year);

    let jp_dt = match &parse_datetime(&jp_input, &options).unwrap()[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => panic!("Expected Absolute"),
    };
    let ymd_dt = match &parse_datetime(&ymd_input, &options).unwrap()[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => panic!("Expected Absolute"),
    };
    let mmdd_space_dt = match &parse_datetime("0316 2100", &options).unwrap()[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => panic!("Expected Absolute"),
    };
    let mmdd_dt = match &parse_datetime("03162100", &options).unwrap()[0] {
        ParsedDateTime::Absolute(dt) => *dt,
        _ => panic!("Expected Absolute"),
    };

    assert_eq!(jp_dt, ymd_dt);
    assert_eq!(jp_dt, md_dt);
    assert_eq!(jp_dt, mmdd_space_dt);
    assert_eq!(jp_dt, mmdd_dt);
}

#[test]
fn test_flag_boundary_japanese_requires_flag() {
    let options = DateTimeParseOptions {
        flags: DateTimeParseFlags::TIME_ONLY,
        timezone: chrono_tz::Asia::Tokyo,
        relative_base: Some(RelativeBase::Time(
            NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
        )),
        default_time: None,
        allow_multiple: false,
        max_count: 1,
    };

    // 日本語フラグを外しているため拒否
    assert!(parse_datetime("午後9時半", &options).is_err());
}

#[test]
fn test_flag_boundary_numeric_requires_flag() {
    let options = DateTimeParseOptions {
        flags: DateTimeParseFlags::TIME_ONLY,
        timezone: chrono_tz::Asia::Tokyo,
        relative_base: Some(RelativeBase::Time(
            NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
        )),
        default_time: None,
        allow_multiple: false,
        max_count: 1,
    };

    // 数字パターンフラグを外しているため拒否
    assert!(parse_datetime("1230", &options).is_err());
}

#[test]
fn test_quest_departure_still_rejects_relative() {
    let options = DateTimeParseOptions::for_quest_departure(chrono_tz::Asia::Tokyo);
    assert!(parse_datetime("1時間後", &options).is_err());
}

#[test]
fn test_multiple_input_limit() {
    let base = Utc::now();
    let options = DateTimeParseOptions::for_dismissal_time(chrono_tz::Asia::Tokyo, base);

    assert!(parse_datetime("1時間前, 21:00, 2日前", &options).is_ok());
    assert!(parse_datetime("1時間前, 21:00, 2日前, 3日前", &options).is_err());
}

#[test]
fn test_dismissal_absolute_range_check() {
    let timezone = chrono_tz::Asia::Tokyo;
    let base = timezone
        .with_ymd_and_hms(2026, 3, 20, 21, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let options = DateTimeParseOptions::for_dismissal_time(timezone, base);
    let max_days = std::env::var("DISMISSAL_MAX_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(7);

    // 出発より後は拒否
    assert!(parse_datetime("2026/03/21 21:00", &options).is_err());

    // 許容範囲内は許可
    assert!(parse_datetime("2026/03/19 20:00", &options).is_ok());

    // 許容日数超過は拒否
    let too_old = base - Duration::days(max_days + 1);
    let too_old_local = too_old.with_timezone(&timezone);
    let input = too_old_local.format("%Y/%m/%d %H:%M").to_string();
    assert!(parse_datetime(&input, &options).is_err());
}
