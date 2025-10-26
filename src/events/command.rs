use crate::types::{AppError, PoiseData};

#[allow(dead_code)]
pub fn commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::slash;
    vec![
        slash::recruit_new::recruit(),
        slash::recruit_cancel::cancel(),
        slash::help::help(),
        slash::environ_load::environ_load(),
        slash::gspread_load::gspread_load(),
        slash::gspread_push::gspread_push(),
        slash::gspread_regist::gspread_regist(),
        slash::gspread_global_load::gspread_global_load(),
        slash::gspread_global_push::gspread_global_push(),
    ]
}
