use crate::types::{AppError, PoiseData};

pub fn commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::slash;
    vec![
        slash::recruit_new::recruit(),
        slash::recruit_cancel::cancel(),
        slash::help::help(),
        slash::environ_load::environ_load(),
    ]
}
