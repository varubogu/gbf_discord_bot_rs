pub mod battle_type;
pub use battle_type::BattleType;

pub type PoiseData = ();
pub type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;
