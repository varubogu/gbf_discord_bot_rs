use crate::types::{AppError, Result};
use chrono::NaiveTime;

/// 時刻パースサービス
///
/// 時刻文字列（HH:MM形式）の解析を担当するサービス。
pub struct TimeParserService;

impl TimeParserService {
    pub fn new() -> Self {
        Self
    }

    /// 時刻文字列をパース（HH:MM形式 → NaiveTime）
    ///
    /// # 引数
    /// - `time_str`: 時刻文字列（例: "22:00", "09:30"）
    ///
    /// # 戻り値
    /// - 成功時: NaiveTime
    /// - 失敗時: AppError::Validation
    ///
    /// # 例
    /// ```
    /// let service = TimeParserService::new();
    /// let time = service.parse_time_string("22:00").unwrap();
    /// assert_eq!(time.hour(), 22);
    /// assert_eq!(time.minute(), 0);
    /// ```
    pub fn parse_time_string(&self, time_str: &str) -> Result<NaiveTime> {
        let parts: Vec<&str> = time_str.split(':').collect();

        if parts.len() != 2 {
            return Err(AppError::Business {
                message: format!("無効な時刻形式です: {time_str}（HH:MM形式で指定してください）"),
            });
        }

        let hour = parts[0].parse::<u32>().map_err(|_| {
            AppError::Business {
                message: format!("無効な時刻です: {time_str}"),
            }
        })?;

        let minute = parts[1].parse::<u32>().map_err(|_| {
            AppError::Business {
                message: format!("無効な時刻です: {time_str}"),
            }
        })?;

        NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| {
            AppError::Business {
                message: format!("無効な時刻です: {time_str}"),
            }
        })
    }
}

impl Default for TimeParserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;

    #[test]
    fn test_parse_time_valid() {
        let service = TimeParserService::new();
        let time = service.parse_time_string("22:00").unwrap();
        assert_eq!(time.hour(), 22);
        assert_eq!(time.minute(), 0);
    }

    #[test]
    fn test_parse_time_morning() {
        let service = TimeParserService::new();
        let time = service.parse_time_string("09:30").unwrap();
        assert_eq!(time.hour(), 9);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_parse_time_midnight() {
        let service = TimeParserService::new();
        let time = service.parse_time_string("00:00").unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 0);
    }

    #[test]
    fn test_parse_time_invalid_format_no_colon() {
        let service = TimeParserService::new();
        let result = service.parse_time_string("2200");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_invalid_format_too_many_parts() {
        let service = TimeParserService::new();
        let result = service.parse_time_string("22:00:00");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_invalid_hour() {
        let service = TimeParserService::new();
        let result = service.parse_time_string("25:00");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_invalid_minute() {
        let service = TimeParserService::new();
        let result = service.parse_time_string("22:60");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_non_numeric() {
        let service = TimeParserService::new();
        let result = service.parse_time_string("AB:CD");
        assert!(result.is_err());
    }
}
