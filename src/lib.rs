pub mod constants;
pub mod errors;
pub mod events;
pub mod facades;
pub mod gateway;
pub mod infrastructure;
pub mod models;
pub mod presenter;
pub mod repository;
pub mod services;
pub mod types;
pub mod utils;

// Library interface for gbf_discord_bot_rs
// This exposes modules for use by examples and external code

// Initialize rust-i18n at the library root
rust_i18n::i18n!("locales");

// Test utilities for integration tests
pub mod test_utils {
    use crate::infrastructure::database::connection::connection_manager::is_database_available;
    use crate::services::unified_datetime_parser::{
        DateTimeParseOptions, ParsedDateTime, parse_datetime,
    };
    use chrono::{DateTime, Local, Timelike};
    use chrono_tz::Tz;

    /// Test utility to check database availability
    pub fn check_database_availability() -> (bool, Vec<String>) {
        is_database_available()
    }

    /// Test utility to get default expiry date (今日21:00)
    pub fn get_default_expiry_date() -> DateTime<Local> {
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

    /// Test utility to parse event date using unified_datetime_parser
    pub fn parse_event_date(date_str: &str) -> Result<DateTime<Local>, String> {
        let trimmed_input = date_str.trim();
        if trimmed_input.is_empty() {
            return Ok(get_default_expiry_date());
        }

        // デフォルトタイムゾーンとしてAsia/Tokyoを使用
        let timezone: Tz = "Asia/Tokyo".parse().unwrap();
        let options = DateTimeParseOptions::for_quest_departure(timezone);

        let results = parse_datetime(trimmed_input, &options)
            .map_err(|e| format!("Failed to parse date string '{trimmed_input}': {e}"))?;

        if results.is_empty() {
            return Err(format!("No valid date parsed from '{trimmed_input}'"));
        }

        // 最初の結果を使用
        match &results[0] {
            ParsedDateTime::Absolute(dt) => {
                // unified_datetime_parserはAsia/Tokyoでパースしたものを
                // UTCに変換して返すため、Asia/Tokyoに戻してからLocalに変換
                let tokyo_tz: Tz = "Asia/Tokyo".parse().unwrap();
                let tokyo_dt = dt.with_timezone(&tokyo_tz);
                Ok(tokyo_dt.with_timezone(&Local))
            }
            ParsedDateTime::Time(time) => {
                // 時刻のみの場合、今日または明日の日付を設定
                let now = Local::now();
                let today_at_time = now
                    .date_naive()
                    .and_time(*time)
                    .and_local_timezone(Local)
                    .single()
                    .unwrap_or(now);

                if today_at_time > now {
                    Ok(today_at_time)
                } else {
                    // 過去の時刻なら明日
                    let tomorrow = now + chrono::Duration::days(1);
                    Ok(tomorrow
                        .date_naive()
                        .and_time(*time)
                        .and_local_timezone(Local)
                        .single()
                        .unwrap_or(now))
                }
            }
            ParsedDateTime::Relative { .. } => {
                Err("Relative time not supported in test utility".to_string())
            }
        }
    }
}
