use crate::types::{PoiseData, PoiseError};

pub fn commands() -> Vec<poise::Command<PoiseData, PoiseError>> {
    use crate::events::interactions::command_interactions::slash;
    vec![
        slash::recruit::recruit(),
        slash::recruit::cannel(),
        slash::help::help(),
        slash::environ_load::environ_load(),
    ]
}
