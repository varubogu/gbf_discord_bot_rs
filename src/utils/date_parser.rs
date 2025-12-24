use chrono::{DateTime, Local, Timelike};
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
        Ok(parsed_datetime) => Ok(parsed_datetime.with_timezone(&Local)),
        Err(e) => {
            error!("Failed to parse date string '{}': {}", trimmed_input, e);
            Err(format!(
                "Failed to parse date string '{trimmed_input}': {e}"
            ))
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
    use chrono::{Datelike, Duration};

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
        // Test a valid time format "21:00" - supported by dateparser
        let result = parse_event_date("21:00").await.unwrap();
        let now = Local::now();

        // The time should be set to 21:00
        assert_eq!(result.hour(), 21);
        assert_eq!(result.minute(), 0);

        // The date should be today or tomorrow depending on current time
        if now.hour() < 21 {
            assert_eq!(result.day(), now.day());
        } else {
            let tomorrow = now + chrono::Duration::days(1);
            assert_eq!(result.day(), tomorrow.day());
        }
    }

    #[tokio::test]
    async fn test_parse_event_date_valid_format() {
        // Test a valid ISO date format supported by dateparser
        let result = parse_event_date("2025-12-25 15:30").await.unwrap();

        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_date() {
        // Test an invalid date (e.g., February 30) - dateparser returns error
        let result = parse_event_date("2/30 12:00").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_time() {
        // Test an invalid time (e.g., 25:70) - dateparser returns error
        let result = parse_event_date("5/5 25:70").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_event_date_invalid_format() {
        // Test completely invalid format - should return error
        let result = parse_event_date("invalid date format").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse date string"));
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
        assert!(result.unwrap_err().contains("Failed to parse date string"));
    }

    #[tokio::test]
    async fn test_parse_time_only_colon_format() {
        // Test time-only format "HH:MM" - supported by dateparser
        let now = Local::now();
        let result = parse_event_date("15:30").await.unwrap();

        // Check that the time is correctly parsed to 15:30
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);

        // The date should be either today or tomorrow
        // We don't make strict assumptions about which one, as dateparser's behavior
        // can vary based on the current time and timezone
        let result_date = result.date_naive();
        let today = now.date_naive();
        let tomorrow = (now + Duration::days(1)).date_naive();

        assert!(
            result_date == today || result_date == tomorrow,
            "Result date {} should be either today {} or tomorrow {}",
            result_date,
            today,
            tomorrow
        );
    }

    // ============ English Pattern Tests ============
    // Note: dateparser has limited support for English relative dates.
    // Many natural language formats are not supported by the dateparser crate.
    // We test only the formats that are actually supported.

    #[tokio::test]
    async fn test_english_iso_format() {
        // Test ISO 8601 format which is well supported
        let result = parse_event_date("2025-12-31").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_english_iso_with_time() {
        // Test ISO 8601 format with time
        let result = parse_event_date("2025-12-31 15:30:00").await;
        assert!(result.is_ok());
    }
}
