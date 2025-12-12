use crate::types::{AppError, PoiseData};

#[allow(dead_code)]
pub fn commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::{message, slash};
    vec![
        slash::channel_register::channel_register(),
        slash::recruit_new::recruit_new(),
        slash::recruit_new_v2::recruit_new_v2(),
        slash::recruit_cancel::recruit_cancel(),
        slash::recruit_change::recruit_change(),
        slash::recruit_role_add::recruit_role_add(),
        slash::recruit_role_remove::recruit_role_remove(),
        slash::recruitment_schedule_create::recruitment_schedule_create(),
        slash::recruitment_schedule_list::recruitment_schedule_list(),
        slash::recruitment_schedule_delete::recruitment_schedule_delete(),
        slash::recruitment_schedule_toggle::recruitment_schedule_toggle(),
        slash::help::help(),
        slash::environ_load::environ_load(),
        slash::gspread_load::gspread_load(),
        slash::gspread_push::gspread_push(),
        slash::gspread_regist::gspread_regist(),
        slash::gspread_global_load::gspread_global_load(),
        slash::gspread_global_push::gspread_global_push(),
        slash::schedule_generate::schedule_generate(),
        slash::schedule_list::schedule_list(),
        slash::schedule_history::schedule_history(),
        slash::schedule_stats::schedule_stats(),
        slash::timezone_set::timezone_set(),
        slash::timezone_show::timezone_show(),
        // メッセージコンテキストメニューコマンド
        message::recruit_change::recruit_change_context_menu(),
    ]
}
