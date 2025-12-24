/// YAML メッセージローダー
///
/// rust-i18n の t! マクロを使用して、各メッセージIDに対応する翻訳を取得します。
/// コンパイル時に最適化され、実行時のオーバーヘッドはほぼゼロです。
use rust_i18n::t;

/// メッセージIDとロケールからYAMLメッセージを取得
///
/// # 引数
/// * `message_id` - メッセージID（例: "timezone.show_current"）
/// * `locale` - ロケール（"ja" または "en"）
///
/// # 戻り値
/// メッセージが見つかった場合は Some(String)、見つからない場合は None
pub fn get_yaml_message(message_id: &str, locale: &str) -> Option<String> {
    match message_id {
        // Common messages
        "common.success" => Some(t!("common.success", locale = locale).to_string()),
        "common.error" => Some(t!("common.error", locale = locale).to_string()),
        "common.warning" => Some(t!("common.warning", locale = locale).to_string()),
        "common.info" => Some(t!("common.info", locale = locale).to_string()),
        "common.yes" => Some(t!("common.yes", locale = locale).to_string()),
        "common.no" => Some(t!("common.no", locale = locale).to_string()),
        "common.cancel" => Some(t!("common.cancel", locale = locale).to_string()),
        "common.confirm" => Some(t!("common.confirm", locale = locale).to_string()),
        "common.loading" => Some(t!("common.loading", locale = locale).to_string()),
        "common.unknown" => Some(t!("common.unknown", locale = locale).to_string()),

        // Battle recruitment messages
        "battle_recruitment.title" => Some(t!("battle_recruitment.title", locale = locale).to_string()),
        "battle_recruitment.new_recruitment" => Some(t!("battle_recruitment.new_recruitment", locale = locale).to_string()),
        "battle_recruitment.recruitment_cancelled" => Some(t!("battle_recruitment.recruitment_cancelled", locale = locale).to_string()),
        "battle_recruitment.recruitment_closed" => Some(t!("battle_recruitment.recruitment_closed", locale = locale).to_string()),
        "battle_recruitment.recruitment_full" => Some(t!("battle_recruitment.recruitment_full", locale = locale).to_string()),
        "battle_recruitment.join_success" => Some(t!("battle_recruitment.join_success", locale = locale).to_string()),
        "battle_recruitment.leave_success" => Some(t!("battle_recruitment.leave_success", locale = locale).to_string()),
        "battle_recruitment.not_found" => Some(t!("battle_recruitment.not_found", locale = locale).to_string()),
        "battle_recruitment.already_joined" => Some(t!("battle_recruitment.already_joined", locale = locale).to_string()),
        "battle_recruitment.not_joined" => Some(t!("battle_recruitment.not_joined", locale = locale).to_string()),

        // Error messages
        "errors.invalid_input" => Some(t!("errors.invalid_input", locale = locale).to_string()),
        "errors.permission_denied" => Some(t!("errors.permission_denied", locale = locale).to_string()),
        "errors.internal_error" => Some(t!("errors.internal_error", locale = locale).to_string()),
        "errors.user_not_found" => Some(t!("errors.user_not_found", locale = locale).to_string()),
        "errors.command_failed" => Some(t!("errors.command_failed", locale = locale).to_string()),
        "errors.env_var_not_set" => Some(t!("errors.env_var_not_set", locale = locale).to_string()),
        "errors.guild_only" => Some(t!("errors.guild_only", locale = locale).to_string()),
        "errors.spreadsheet_not_registered" => Some(t!("errors.spreadsheet_not_registered", locale = locale).to_string()),
        "errors.spreadsheet_config_fetch_failed" => Some(t!("errors.spreadsheet_config_fetch_failed", locale = locale).to_string()),

        // Spreadsheet messages
        "spreadsheet.loading" => Some(t!("spreadsheet.loading", locale = locale).to_string()),
        "spreadsheet.load_success" => Some(t!("spreadsheet.load_success", locale = locale).to_string()),
        "spreadsheet.load_partial_success" => Some(t!("spreadsheet.load_partial_success", locale = locale).to_string()),
        "spreadsheet.load_failed" => Some(t!("spreadsheet.load_failed", locale = locale).to_string()),
        "spreadsheet.registering" => Some(t!("spreadsheet.registering", locale = locale).to_string()),
        "spreadsheet.register_success" => Some(t!("spreadsheet.register_success", locale = locale).to_string()),
        "spreadsheet.register_failed" => Some(t!("spreadsheet.register_failed", locale = locale).to_string()),
        "spreadsheet.pushing" => Some(t!("spreadsheet.pushing", locale = locale).to_string()),
        "spreadsheet.push_success" => Some(t!("spreadsheet.push_success", locale = locale).to_string()),
        "spreadsheet.push_partial_success" => Some(t!("spreadsheet.push_partial_success", locale = locale).to_string()),
        "spreadsheet.push_failed" => Some(t!("spreadsheet.push_failed", locale = locale).to_string()),
        "spreadsheet.global_pushing" => Some(t!("spreadsheet.global_pushing", locale = locale).to_string()),
        "spreadsheet.global_push_success" => Some(t!("spreadsheet.global_push_success", locale = locale).to_string()),
        "spreadsheet.global_push_partial_success" => Some(t!("spreadsheet.global_push_partial_success", locale = locale).to_string()),
        "spreadsheet.global_push_failed" => Some(t!("spreadsheet.global_push_failed", locale = locale).to_string()),

        // Kosenjo messages
        "kosenjo.before_3_days" => Some(t!("kosenjo.before_3_days", locale = locale).to_string()),
        "kosenjo.before_1_day" => Some(t!("kosenjo.before_1_day", locale = locale).to_string()),
        "kosenjo.qualifying_start" => Some(t!("kosenjo.qualifying_start", locale = locale).to_string()),
        "kosenjo.qualifying_end" => Some(t!("kosenjo.qualifying_end", locale = locale).to_string()),
        "kosenjo.qualifying_end_no_interval" => Some(t!("kosenjo.qualifying_end_no_interval", locale = locale).to_string()),
        "kosenjo.main_tournament_before_1_day" => Some(t!("kosenjo.main_tournament_before_1_day", locale = locale).to_string()),
        "kosenjo.main_tournament_day_start" => Some(t!("kosenjo.main_tournament_day_start", locale = locale).to_string()),
        "kosenjo.main_tournament_half_day" => Some(t!("kosenjo.main_tournament_half_day", locale = locale).to_string()),
        "kosenjo.main_tournament_day_end" => Some(t!("kosenjo.main_tournament_day_end", locale = locale).to_string()),
        "kosenjo.main_tournament_end" => Some(t!("kosenjo.main_tournament_end", locale = locale).to_string()),
        "kosenjo.sp_battle_end" => Some(t!("kosenjo.sp_battle_end", locale = locale).to_string()),
        "kosenjo.team_ability_1" => Some(t!("kosenjo.team_ability_1", locale = locale).to_string()),
        "kosenjo.team_ability_2" => Some(t!("kosenjo.team_ability_2", locale = locale).to_string()),

        // Dorebara messages
        "dorebara.start" => Some(t!("dorebara.start", locale = locale).to_string()),
        "dorebara.end" => Some(t!("dorebara.end", locale = locale).to_string()),
        "dorebara.reset" => Some(t!("dorebara.reset", locale = locale).to_string()),
        "dorebara.variant" => Some(t!("dorebara.variant", locale = locale).to_string()),
        "dorebara.last_day" => Some(t!("dorebara.last_day", locale = locale).to_string()),

        // Bot messages
        "bot.mention" => Some(t!("bot.mention", locale = locale).to_string()),
        "bot.mention_six" => Some(t!("bot.mention_six", locale = locale).to_string()),
        "bot.mention_calling" => Some(t!("bot.mention_calling", locale = locale).to_string()),

        // Omikuji messages
        "omikuji.hihi" => Some(t!("omikuji.hihi", locale = locale).to_string()),
        "omikuji.hakyoku" => Some(t!("omikuji.hakyoku", locale = locale).to_string()),
        "omikuji.omega_unit" => Some(t!("omikuji.omega_unit", locale = locale).to_string()),

        // Recruitment messages
        "recruitment.normal" => Some(t!("recruitment.normal", locale = locale).to_string()),
        "recruitment.six_elements" => Some(t!("recruitment.six_elements", locale = locale).to_string()),
        "recruitment.member_full" => Some(t!("recruitment.member_full", locale = locale).to_string()),
        "recruitment.before_5_minutes" => Some(t!("recruitment.before_5_minutes", locale = locale).to_string()),
        "recruitment.start" => Some(t!("recruitment.start", locale = locale).to_string()),
        "recruitment.event_date_label" => Some(t!("recruitment.event_date_label", locale = locale).to_string()),
        "recruitment.date_format" => Some(t!("recruitment.date_format", locale = locale).to_string()),
        "recruitment.element_fire" => Some(t!("recruitment.element_fire", locale = locale).to_string()),
        "recruitment.element_water" => Some(t!("recruitment.element_water", locale = locale).to_string()),
        "recruitment.element_earth" => Some(t!("recruitment.element_earth", locale = locale).to_string()),
        "recruitment.element_wind" => Some(t!("recruitment.element_wind", locale = locale).to_string()),
        "recruitment.element_light" => Some(t!("recruitment.element_light", locale = locale).to_string()),
        "recruitment.element_dark" => Some(t!("recruitment.element_dark", locale = locale).to_string()),
        "recruitment.all_elements" => Some(t!("recruitment.all_elements", locale = locale).to_string()),
        "recruitment.no_participants" => Some(t!("recruitment.no_participants", locale = locale).to_string()),
        "recruitment.leave_all_button" => Some(t!("recruitment.leave_all_button", locale = locale).to_string()),

        // Timezone messages
        "timezone.set_success" => Some(t!("timezone.set_success", locale = locale).to_string()),
        "timezone.show_current" => Some(t!("timezone.show_current", locale = locale).to_string()),

        // Recruit role messages
        "recruit_role.add_success" => Some(t!("recruit_role.add_success", locale = locale).to_string()),
        "recruit_role.remove_success" => Some(t!("recruit_role.remove_success", locale = locale).to_string()),

        // Recruit messages
        "recruit.cancel_already_cancelled" => Some(t!("recruit.cancel_already_cancelled", locale = locale).to_string()),
        "recruit.cancel_message_deleted" => Some(t!("recruit.cancel_message_deleted", locale = locale).to_string()),
        "recruit.cancel_invalid_message" => Some(t!("recruit.cancel_invalid_message", locale = locale).to_string()),
        "recruit.cancel_not_found" => Some(t!("recruit.cancel_not_found", locale = locale).to_string()),
        "recruit.cancel_error" => Some(t!("recruit.cancel_error", locale = locale).to_string()),
        "recruit.change_no_changes" => Some(t!("recruit.change_no_changes", locale = locale).to_string()),
        "recruit.change_success" => Some(t!("recruit.change_success", locale = locale).to_string()),

        // General messages
        "messages.welcome" => Some(t!("messages.welcome", locale = locale).to_string()),
        "messages.help" => Some(t!("messages.help", locale = locale).to_string()),

        // 未知のメッセージID
        _ => None,
    }
}
