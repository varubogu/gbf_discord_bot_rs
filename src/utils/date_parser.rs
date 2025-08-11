use chrono::{Local, NaiveDate, NaiveTime, TimeZone, Datelike, Timelike};
use regex::Regex;
use tracing::error;

/// Parses a date string in the format "MM/DD HH:MM" or similar formats
pub async fn parse_event_date(date_str: &str) -> Result<chrono::DateTime<Local>, String> {
    // Default to today at 21:00 if parsing fails
    let default_date = default_expiry_date().await;
    
    // Simple case: "今日 21:00"
    if date_str == "今日 21:00" {
        return Ok(default_date);
    }
    
    // Try to parse with regex
    let re = Regex::new(r"(\d{1,2})/(\d{1,2})\s+(\d{1,2}):(\d{1,2})").unwrap();
    if let Some(caps) = re.captures(date_str) {
        let month: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(default_date.month());
        let day: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(default_date.day());
        let hour: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(21);
        let minute: u32 = caps.get(4).unwrap().as_str().parse().unwrap_or(0);
        
        let now = Local::now();
        let year = now.year();
        
        // Create date with the current year
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => {
                match NaiveTime::from_hms_opt(hour, minute, 0) {
                    Some(time) => {
                        let naive_datetime = date.and_time(time);
                        match Local.from_local_datetime(&naive_datetime) {
                            chrono::LocalResult::Single(dt) => return Ok(dt),
                            _ => {
                                error!("Error converting to local datetime");
                                return Ok(default_date);
                            }
                        }
                    },
                    None => {
                        error!("Invalid time: {}:{}", hour, minute);
                        return Ok(default_date);
                    }
                }
            },
            None => {
                error!("Invalid date: {}/{}", month, day);
                return Ok(default_date);
            }
        }
    }
    
    // If all parsing fails, return the default
    Ok(default_date)
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
        // Test completely invalid format
        let result = parse_event_date("invalid date format").await.unwrap();
        let default = default_expiry_date().await;
        
        // Should return the default date
        assert_eq!(result.year(), default.year());
        assert_eq!(result.month(), default.month());
        assert_eq!(result.day(), default.day());
        assert_eq!(result.hour(), default.hour());
        assert_eq!(result.minute(), default.minute());
    }
}