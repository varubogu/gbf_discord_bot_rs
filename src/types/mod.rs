pub mod battle_type;
pub use battle_type::BattleType;
use std::sync::Arc;
use crate::utils::database::DatabaseService;

#[derive(Debug)]
pub struct PoiseData {
    pub db: Arc<dyn DatabaseService>,
}
pub type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;
