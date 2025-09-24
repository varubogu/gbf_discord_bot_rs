pub mod constants;
mod events;
mod facades;
pub mod infrastructure;
mod models;
mod repository;
pub mod services;
mod types;
pub mod utils;

// Library interface for gbf_discord_bot_rs
// This exposes modules for use by examples and external code

// Initialize rust-i18n at the library root
rust_i18n::i18n!("locales");
