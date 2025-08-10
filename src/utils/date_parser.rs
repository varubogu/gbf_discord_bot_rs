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