// Library interface for gbf_discord_bot_rs
// This exposes modules for use by examples and external code

// Initialize rust-i18n at the library root
rust_i18n::i18n!("locales");

pub mod events;
pub mod services;
pub mod utils;
pub mod models;
pub mod types;
pub mod repository;
pub mod facades;