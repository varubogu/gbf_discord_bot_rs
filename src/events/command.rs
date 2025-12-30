use crate::types::{AppError, PoiseData};

/// 全コマンド（グローバル + 管理サーバー専用）
#[allow(dead_code)]
pub fn commands() -> Vec<poise::Command<PoiseData, AppError>> {
    let mut all_commands = global_commands();
    all_commands.extend(admin_commands());
    all_commands
}

/// 全サーバーに登録するグローバルコマンド
pub fn global_commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::{message, slash};
    vec![
        slash::channel_register::channel_register(),
        slash::channel_show::channel_show(),
        slash::channel_unregister::channel_unregister(),
        slash::quest_disable::quest_disable(),
        slash::quest_enable::quest_enable(),
        slash::quest_list::quest_list(),
        slash::recruit_new::recruit_new(),
        slash::recruit_new_v2::recruit_new_v2(),
        slash::recruit_cancel::recruit_cancel(),
        slash::recruit_change::recruit_change(),
        slash::recruit_role_add::recruit_role_add(),
        slash::recruit_role_remove::recruit_role_remove(),
        slash::recruit_role_show::recruit_role_show(),
        slash::recruitment_schedule_create::recruitment_schedule_create(),
        slash::recruitment_schedule_list::recruitment_schedule_list(),
        slash::recruitment_schedule_delete::recruitment_schedule_delete(),
        slash::recruitment_schedule_toggle::recruitment_schedule_toggle(),
        slash::help::help(),
        slash::environ_load::environ_load(),
        slash::gspread_load::gspread_load(),
        slash::gspread_push::gspread_push(),
        slash::gspread_regist::gspread_regist(),
        slash::schedule_generate::schedule_generate(),
        slash::schedule_list::schedule_list(),
        slash::schedule_history::schedule_history(),
        slash::schedule_stats::schedule_stats(),
        slash::guild_settings_set::set_guild_settings(),
        slash::guild_settings_show::show_guild_settings(),
        // メッセージコンテキストメニューコマンド
        message::recruit_change::recruit_change_context_menu(),
    ]
}

/// 管理サーバー専用コマンド（特定ギルドにのみ登録）
pub fn admin_commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::slash;
    vec![
        slash::gspread_global_load::gspread_global_load(),
        slash::gspread_global_push::gspread_global_push(),
    ]
}
