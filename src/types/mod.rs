pub(crate) mod app_config;
pub(crate) mod app_error;
pub(crate) mod app_state;
pub(crate) mod battle_type;
pub(crate) mod discord_operation;
pub(crate) mod poise_data;
pub mod transaction;

pub use app_config::AppConfig;
// Re-export all public types for backward compatibility
pub use app_error::{AppError, Result};
pub use app_state::AppState;
pub use discord_operation::{DiscordOperation, DiscordOperationError, DiscordOperationResult};
pub use poise_data::{PoiseContext, PoiseData};
pub use transaction::{DatabaseConnectionTrait, DatabaseTransactionTrait, TransactionManagerTrait};
