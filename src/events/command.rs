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
        slash::channel::channel_register::channel_register(),
        slash::channel::channel_show::channel_show(),
        slash::channel::channel_unregister::channel_unregister(),
        slash::quest::quest_disable::quest_disable(),
        slash::quest::quest_enable::quest_enable(),
        slash::quest::quest_list::quest_list(),
        slash::recruit::recruit_new::recruit_new(),
        slash::recruit::recruit_new_v2::recruit_new_v2(),
        slash::recruit::recruit_cancel::recruit_cancel(),
        slash::recruit::recruit_change::recruit_change(),
        slash::recruit::recruit_role_add::recruit_role_add(),
        slash::recruit::recruit_role_remove::recruit_role_remove(),
        slash::recruit::recruit_role_show::recruit_role_show(),
        slash::recruit::recruitment_schedule_create::recruitment_schedule_create(),
        slash::recruit::recruitment_schedule_list::recruitment_schedule_list(),
        slash::recruit::recruitment_schedule_delete::recruitment_schedule_delete(),
        slash::recruit::recruitment_schedule_toggle::recruitment_schedule_toggle(),
        slash::util::help::help(),
        slash::util::environ_load::environ_load(),
        slash::gspread::gspread_load::gspread_load(),
        slash::gspread::gspread_push::gspread_push(),
        slash::gspread::gspread_register::gspread_register(),
        slash::schedule::schedule_generate::schedule_generate(),
        slash::schedule::schedule_list::schedule_list(),
        slash::schedule::schedule_history::schedule_history(),
        slash::schedule::schedule_stats::schedule_stats(),
        slash::guild_settings::guild_settings_set::guild_settings_set(),
        slash::guild_settings::guild_settings_show::guild_settings_show(),
        // 自動募集コマンド
        slash::auto_recruit::category_register::auto_recruit_category_register(),
        slash::auto_recruit::category_unregister::auto_recruit_category_unregister(),
        slash::auto_recruit::days_change::auto_recruit_days_change(),
        slash::auto_recruit::status::auto_recruit_status(),
        // メッセージコンテキストメニューコマンド
        message::recruit_change::recruit_change_context_menu(),
    ]
}

/// 管理サーバー専用コマンド（特定ギルドにのみ登録）
pub fn admin_commands() -> Vec<poise::Command<PoiseData, AppError>> {
    use crate::events::interactions::command_interactions::slash;
    vec![
        slash::gspread::gspread_global_load::gspread_global_load(),
        slash::gspread::gspread_global_push::gspread_global_push(),
    ]
}
