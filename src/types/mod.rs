pub(crate) mod battle_type;

#[derive(Debug)]
pub struct PoiseData {}
pub type PoiseError = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;
