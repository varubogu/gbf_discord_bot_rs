use chrono::{DateTime, Local, Datelike, Timelike, Duration};
use dateparser;
use tracing::error;

/// Parses a date string using the dateparser crate
/// 
/// Requirements from issue:
/// - Use dateparser crate for parsing
/// - For empty input, treat it as "今日21:00" (today 21:00) and continue parsing
/// - Return error if parsing fails
pub async fn parse_event_date(date_str: &str) -> Result<DateTime<Local>, String> {
    let trimmed_input = date_str.trim();
    if trimmed_input.is_empty() {
        return Ok(default_expiry_date().await);
    }
    
    // Use dateparser crate to parse the input
    match dateparser::parse(trimmed_input) {
        Ok(parsed_datetime) => {
            Ok(parsed_datetime.with_timezone(&Local))
        }
        Err(e) => {
            error!("Failed to parse date string '{}': {}", trimmed_input, e);
            Err(format!("Failed to parse date string '{}': {}", trimmed_input, e))
        }
    }
}

/// Returns the default expiry date (today at 21:00)
pub async fn default_expiry_date() -> chrono::DateTime<Local> {
    let now = Local::now();
    now.with_hour(21)
        .unwrap_or(now)
        .with_minute(0)
        .unwrap_or(now)
        .with_second(0)
        .unwrap_or(now)
        .with_nanosecond(0)
        .unwrap_or(now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_expiry_date() {
        let result = default_expiry_date().await;
        let now = Local::now();
        
        // Check that the date is today
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        
        // Check that the time is 21:00:00
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 0);
        assert_eq!(result.second(), 0);
    }

    #[tokio::test]
    async fn test_parse_event_date_default_case() {
        // Test the "今日 21:00" case
        let result = parse_event_date("今日 21:00").await.unwrap();
        let default = default_expiry_date().await;
        
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_event_date_valid_format() {
        // Test a valid date format "MM/DD HH:MM"
        let result = parse_event_date("12/25 15:30").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_date() {
        // Test an invalid date (e.g., February 30)
        let result = parse_event_date("2/30 12:00").await.unwrap();
        let default = default_expiry_date().await;
        
        // Should return the default date
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), default.hour());
        assert_eq!(result.minute(), default.minute());
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_time() {
        // Test an invalid time (e.g., 25:70)
        let result = parse_event_date("5/5 25:70").await.unwrap();
        let default = default_expiry_date().await;
        
        // Should return the default date
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), default.hour());
        assert_eq!(result.minute(), default.minute());
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_format() {
        // Test completely invalid format - should return error
        let result = parse_event_date("invalid date format").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unable to parse date string"));
    }

    #[tokio::test]
    async fn test_parse_event_date_empty_string() {
        // Test empty string - should return default value
        let result = parse_event_date("").await.unwrap();
        let default = default_expiry_date().await;
        
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), default.hour());
        assert_eq!(result.minute(), default.minute());
    }

    #[tokio::test]
    async fn test_parse_event_date_whitespace_only() {
        // Test whitespace-only input - should return default value
        let result = parse_event_date("   \t\n  ").await.unwrap();
        let default = default_expiry_date().await;
        
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), default.hour());
        assert_eq!(result.minute(), default.minute());
    }

    #[tokio::test]
    async fn test_parse_event_date_unparseable_content() {
        // Test unparseable content with actual text - should return error
        let result = parse_event_date("random text 123").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unable to parse date string"));
    }

    #[tokio::test]
    async fn test_parse_event_date_only_slash_format() {
        // Test date-only format "MM/DD"
        let result = parse_event_date("12/25").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_event_date_only_japanese_format() {
        // Test date-only Japanese format "MM月DD日"
        let result = parse_event_date("12月25日").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_time_only_colon_format() {
        // Test time-only format "HH:MM"
        let now = Local::now();
        let result = parse_event_date("15:30").await.unwrap();
        
        // Should be today at 15:30 if it hasn't passed, or tomorrow if it has
        if now.hour() < 15 || (now.hour() == 15 && now.minute() < 30) {
            // Time hasn't passed today
            assert_eq!(result.day(), now.day());
        } else {
            // Time has passed, should be tomorrow
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_time_only_japanese_format() {
        // Test time-only Japanese format "HH時MM分"
        let now = Local::now();
        let result = parse_event_date("15時30分").await.unwrap();
        
        // Should be today at 15:30 if it hasn't passed, or tomorrow if it has
        if now.hour() < 15 || (now.hour() == 15 && now.minute() < 30) {
            // Time hasn't passed today
            assert_eq!(result.day(), now.day());
        } else {
            // Time has passed, should be tomorrow
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_time_only_half_hour_format() {
        // Test time-only Japanese format "HH時半"
        let now = Local::now();
        let result = parse_event_date("15時半").await.unwrap();
        
        // Should be today at 15:30 if it hasn't passed, or tomorrow if it has
        if now.hour() < 15 || (now.hour() == 15 && now.minute() < 30) {
            // Time hasn't passed today
            assert_eq!(result.day(), now.day());
        } else {
            // Time has passed, should be tomorrow
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_relative_date_today() {
        // Test relative date "今日"
        let result = parse_event_date("今日").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_relative_date_tomorrow() {
        // Test relative date "明日"
        let result = parse_event_date("明日").await.unwrap();
        let tomorrow = Local::now() + Duration::days(1);
        
        assert_eq!(result.year(), tomorrow.year());
        assert_eq!(result.month(), tomorrow.month());
        assert_eq!(result.day(), tomorrow.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_relative_date_days_after() {
        // Test relative date "3日後"
        let result = parse_event_date("3日後").await.unwrap();
        let target_date = Local::now() + Duration::days(3);
        
        assert_eq!(result.year(), target_date.year());
        assert_eq!(result.month(), target_date.month());
        assert_eq!(result.day(), target_date.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_parse_relative_date_with_time_today() {
        // Test relative date with time "今日 15:30"
        let result = parse_event_date("今日 15:30").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_relative_date_with_time_tomorrow() {
        // Test relative date with time "明日22時30分"
        let result = parse_event_date("明日22時30分").await.unwrap();
        let tomorrow = Local::now() + Duration::days(1);
        
        assert_eq!(result.year(), tomorrow.year());
        assert_eq!(result.month(), tomorrow.month());
        assert_eq!(result.day(), tomorrow.day());
        assert_eq!(result.hour(), 22);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_relative_date_with_time_days_after() {
        // Test relative date with time "2日後 15時半"
        let result = parse_event_date("2日後 15時半").await.unwrap();
        let target_date = Local::now() + Duration::days(2);
        
        assert_eq!(result.year(), target_date.year());
        assert_eq!(result.month(), target_date.month());
        assert_eq!(result.day(), target_date.day());
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_date_time_japanese_format() {
        // Test Japanese date and time format "12月25日 15時30分"
        let result = parse_event_date("12月25日 15時30分").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_date_time_mixed_format() {
        // Test mixed format "12月25日 15:30"
        let result = parse_event_date("12月25日 15:30").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    // ============ English Pattern Tests ============
    
    #[tokio::test]
    async fn test_english_time_am_pm() {
        // Test English time with AM/PM
        let now = Local::now();
        let result = parse_event_date("2:30 PM").await.unwrap();
        
        // Should be today at 14:30 if it hasn't passed, or tomorrow if it has
        if now.hour() < 14 || (now.hour() == 14 && now.minute() < 30) {
            assert_eq!(result.day(), now.day());
        } else {
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 14);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_english_time_half_past() {
        // Test English "half past" format
        let now = Local::now();
        let result = parse_event_date("half past 3").await.unwrap();
        
        // Should be today at 3:30 if it hasn't passed, or tomorrow if it has
        if now.hour() < 3 || (now.hour() == 3 && now.minute() < 30) {
            assert_eq!(result.day(), now.day());
        } else {
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 3);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_english_time_quarter_past() {
        // Test English "quarter past" format
        let now = Local::now();
        let result = parse_event_date("quarter past 5").await.unwrap();
        
        // Should be today at 5:15 if it hasn't passed, or tomorrow if it has
        if now.hour() < 5 || (now.hour() == 5 && now.minute() < 15) {
            assert_eq!(result.day(), now.day());
        } else {
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 5);
        assert_eq!(result.minute(), 15);
    }

    #[tokio::test]
    async fn test_english_time_quarter_to() {
        // Test English "quarter to" format
        let now = Local::now();
        let result = parse_event_date("quarter to 6").await.unwrap();
        
        // Should be today at 5:45 if it hasn't passed, or tomorrow if it has
        if now.hour() < 5 || (now.hour() == 5 && now.minute() < 45) {
            assert_eq!(result.day(), now.day());
        } else {
            let tomorrow = now + Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
        assert_eq!(result.hour(), 5);
        assert_eq!(result.minute(), 45);
    }

    #[tokio::test]
    async fn test_english_date_month_name() {
        // Test English date with month name "Dec 31"
        let result = parse_event_date("Dec 31").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 31);
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_date_full_month_name() {
        // Test English date with full month name "January 15th"
        let result = parse_event_date("January 15th").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_date_day_month_format() {
        // Test English date "31st Dec" format
        let result = parse_event_date("31st Dec").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 31);
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_relative_today() {
        // Test English relative date "today"
        let result = parse_event_date("today").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_relative_tomorrow() {
        // Test English relative date "tomorrow"
        let result = parse_event_date("tomorrow").await.unwrap();
        let tomorrow = Local::now() + Duration::days(1);
        
        assert_eq!(result.year(), tomorrow.year());
        assert_eq!(result.month(), tomorrow.month());
        assert_eq!(result.day(), tomorrow.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_relative_days_later() {
        // Test English relative date "3 days later"
        let result = parse_event_date("3 days later").await.unwrap();
        let target_date = Local::now() + Duration::days(3);
        
        assert_eq!(result.year(), target_date.year());
        assert_eq!(result.month(), target_date.month());
        assert_eq!(result.day(), target_date.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_relative_in_days() {
        // Test English relative date "in 5 days"
        let result = parse_event_date("in 5 days").await.unwrap();
        let target_date = Local::now() + Duration::days(5);
        
        assert_eq!(result.year(), target_date.year());
        assert_eq!(result.month(), target_date.month());
        assert_eq!(result.day(), target_date.day());
        assert_eq!(result.hour(), 21); // Default time
        assert_eq!(result.minute(), 0);
    }

    #[tokio::test]
    async fn test_english_relative_with_time_am_pm() {
        // Test English relative date with time "today 3:30 PM"
        let result = parse_event_date("today 3:30 PM").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_english_relative_with_time_quarter_past() {
        // Test English relative date with time "tomorrow quarter past 2"
        let result = parse_event_date("tomorrow quarter past 2").await.unwrap();
        let tomorrow = Local::now() + Duration::days(1);
        
        assert_eq!(result.year(), tomorrow.year());
        assert_eq!(result.month(), tomorrow.month());
        assert_eq!(result.day(), tomorrow.day());
        assert_eq!(result.hour(), 2);
        assert_eq!(result.minute(), 15);
    }

    #[tokio::test]
    async fn test_english_date_time_am_pm() {
        // Test English date and time "Dec 25 2:30 PM"
        let result = parse_event_date("Dec 25 2:30 PM").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 14);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_english_date_time_day_month_format() {
        // Test English date and time "25th Dec 10:15 AM"
        let result = parse_event_date("25th Dec 10:15 AM").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 10);
        assert_eq!(result.minute(), 15);
    }

    #[tokio::test]
    async fn test_english_date_time_full_month() {
        // Test English date and time "January 1st 11:45 PM"
        let result = parse_event_date("January 1st 11:45 PM").await.unwrap();
        let now = Local::now();
        
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 1);
        assert_eq!(result.hour(), 23);
        assert_eq!(result.minute(), 45);
    }

}