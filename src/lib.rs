// Library file for GBF Discord Bot
// This exposes modules for testing and potential library usage

pub mod events;
pub mod services;
pub mod utils;
pub mod models;
pub mod config;

// Re-export commonly used items for convenience
pub use services::database::Database;
pub use services::battle::{BattleType, RecruitmentService};
pub use events::handler::EventHandler;
pub use utils::{date_parser, discord_helper};