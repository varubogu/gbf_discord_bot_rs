use gbf_discord_bot_rs::utils::date_parser;
use chrono::{Local, Datelike, Timelike};

#[tokio::test]
async fn test_date_parser_integration() {
    // Test that the default expiry date is correctly set to today at 21:00
    let default_date = date_parser::default_expiry_date().await;
    let now = Local::now();
    
    // Check that the date is today
    assert_eq!(default_date.year(), now.year());
    assert_eq!(default_date.month(), now.month());
    assert_eq!(default_date.day(), now.day());
    
    // Check that the time is 21:00:00
    assert_eq!(default_date.hour(), 21);
    assert_eq!(default_date.minute(), 0);
    assert_eq!(default_date.second(), 0);
    
    // Test parsing a date string
    let parsed_date = date_parser::parse_event_date("12/25 15:30").await.unwrap();
    
    // Check that the parsed date is correct
    assert_eq!(parsed_date.year(), now.year());
    assert_eq!(parsed_date.month(), 12);
    assert_eq!(parsed_date.day(), 25);
    assert_eq!(parsed_date.hour(), 15);
    assert_eq!(parsed_date.minute(), 30);
}