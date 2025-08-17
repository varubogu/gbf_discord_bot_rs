pub mod battle_type;
use crate::infrastructure::database::DatabaseService;
pub use battle_type::BattleType;

#[derive(Debug)]
pub struct PoiseData {}
pub type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;
