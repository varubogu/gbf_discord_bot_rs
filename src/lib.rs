pub mod constants;
pub mod errors;
pub mod events;
pub mod facades;
pub mod infrastructure;
mod models;
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
    use crate::utils::date_parser;

    /// Test utility to check database availability
    pub fn check_database_availability() -> (bool, Vec<String>) {
        is_database_available()
    }

    /// Test utility to get default expiry date
    pub async fn get_default_expiry_date() -> chrono::DateTime<chrono::Local> {
        date_parser::default_expiry_date().await
    }

    /// Test utility to parse event date
    pub async fn parse_event_date(
        date_str: &str,
    ) -> Result<chrono::DateTime<chrono::Local>, String> {
        date_parser::parse_event_date(date_str).await
    }
}
