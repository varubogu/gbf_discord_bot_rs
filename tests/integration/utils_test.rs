use chrono::{Datelike, Local, Timelike};
use gbf_discord_bot_rs::test_utils;

#[tokio::test]
async fn test_date_parser_integration() {
    // Test that the default expiry date is correctly set to today at 21:00
    let default_date = test_utils::get_default_expiry_date();
    let now = Local::now();

    // Check that the date is today
    assert_eq!(default_date.year(), now.year());
    assert_eq!(default_date.month(), now.month());
    assert_eq!(default_date.day(), now.day());

    // Check that the time is 21:00:00
    assert_eq!(default_date.hour(), 21);
    assert_eq!(default_date.minute(), 0);
    assert_eq!(default_date.second(), 0);

    // Test parsing a date string using unified_datetime_parser format
    let parsed_date = test_utils::parse_event_date("2025/12/25 15:30").unwrap();

    // unified_datetime_parserは"2025-12-25"ではなく"2025/12/25"形式を想定
    // Asia/Tokyoタイムゾーンでパースされ、環境のLocalタイムゾーンに変換される
    // CI環境はUTCなので、JST 15:30はUTC 06:30になる
    assert_eq!(parsed_date.year(), 2025);
    assert_eq!(parsed_date.month(), 12);
    assert_eq!(parsed_date.day(), 25);

    // ローカルタイムゾーンがUTCの場合、JST 15:30 = UTC 06:30
    // ローカルタイムゾーンがJSTの場合、JST 15:30 = JST 15:30
    let hour = parsed_date.hour();
    assert!(hour == 15 || hour == 6, "Expected hour to be 15 (JST) or 6 (UTC), got {}", hour);
    assert_eq!(parsed_date.minute(), 30);
}
