/// メッセージキー定数定義
///
/// YAMLファイル内のキーを定数として定義する。
/// これらの定数は `MessageTextId::as_str()` と `yaml_loader.rs` の両方から参照される。
/// 単一の情報源(Single Source of Truth)として機能する。
pub mod keys {
    // Common messages
    pub const COMMON_SUCCESS: &str = "common.success";
    pub const COMMON_ERROR: &str = "common.error";
    pub const COMMON_ERROR_PREFIX: &str = "common.error_prefix";
    pub const COMMON_WARNING: &str = "common.warning";
    pub const COMMON_INFO: &str = "common.info";
    pub const COMMON_YES: &str = "common.yes";
    pub const COMMON_NO: &str = "common.no";
    pub const COMMON_CANCEL: &str = "common.cancel";
    pub const COMMON_CONFIRM: &str = "common.confirm";
    pub const COMMON_LOADING: &str = "common.loading";
    pub const COMMON_UNKNOWN: &str = "common.unknown";

    // Recruitment UI messages
    pub const RECRUITMENT_UI_TITLE: &str = "recruitment.ui.title";
    pub const RECRUITMENT_UI_NEW_RECRUITMENT: &str = "recruitment.ui.new_recruitment";
    pub const RECRUITMENT_UI_RECRUITMENT_CANCELLED: &str = "recruitment.ui.recruitment_cancelled";
    pub const RECRUITMENT_UI_RECRUITMENT_CLOSED: &str = "recruitment.ui.recruitment_closed";
    pub const RECRUITMENT_UI_RECRUITMENT_FULL: &str = "recruitment.ui.recruitment_full";
    pub const RECRUITMENT_UI_JOIN_SUCCESS: &str = "recruitment.ui.join_success";
    pub const RECRUITMENT_UI_LEAVE_SUCCESS: &str = "recruitment.ui.leave_success";
    pub const RECRUITMENT_UI_NOT_FOUND: &str = "recruitment.ui.not_found";
    pub const RECRUITMENT_UI_ALREADY_JOINED: &str = "recruitment.ui.already_joined";
    pub const RECRUITMENT_UI_NOT_JOINED: &str = "recruitment.ui.not_joined";

    // Error messages
    pub const ERRORS_INVALID_INPUT: &str = "errors.invalid_input";
    pub const ERRORS_PERMISSION_DENIED: &str = "errors.permission_denied";
    pub const ERRORS_INTERNAL_ERROR: &str = "errors.internal_error";
    pub const ERRORS_USER_NOT_FOUND: &str = "errors.user_not_found";
    pub const ERRORS_COMMAND_FAILED: &str = "errors.command_failed";
    pub const ERRORS_ENV_VAR_NOT_SET: &str = "errors.env_var_not_set";
    pub const ERRORS_GUILD_ONLY: &str = "errors.guild_only";
    pub const ERRORS_SPREADSHEET_NOT_REGISTERED: &str = "errors.spreadsheet_not_registered";
    pub const ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED: &str =
        "errors.spreadsheet_config_fetch_failed";

    // Spreadsheet messages
    pub const SPREADSHEET_LOADING: &str = "spreadsheet.loading";
    pub const SPREADSHEET_LOAD_SUCCESS: &str = "spreadsheet.load_success";
    pub const SPREADSHEET_LOAD_PARTIAL_SUCCESS: &str = "spreadsheet.load_partial_success";
    pub const SPREADSHEET_LOAD_FAILED: &str = "spreadsheet.load_failed";
    pub const SPREADSHEET_REGISTERING: &str = "spreadsheet.registering";
    pub const SPREADSHEET_REGISTER_SUCCESS: &str = "spreadsheet.register_success";
    pub const SPREADSHEET_REGISTER_FAILED: &str = "spreadsheet.register_failed";
    pub const SPREADSHEET_PUSHING: &str = "spreadsheet.pushing";
    pub const SPREADSHEET_PUSH_SUCCESS: &str = "spreadsheet.push_success";
    pub const SPREADSHEET_PUSH_PARTIAL_SUCCESS: &str = "spreadsheet.push_partial_success";
    pub const SPREADSHEET_PUSH_FAILED: &str = "spreadsheet.push_failed";
    pub const SPREADSHEET_GLOBAL_PUSHING: &str = "spreadsheet.global_pushing";
    pub const SPREADSHEET_GLOBAL_PUSH_SUCCESS: &str = "spreadsheet.global_push_success";
    pub const SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS: &str =
        "spreadsheet.global_push_partial_success";
    pub const SPREADSHEET_GLOBAL_PUSH_FAILED: &str = "spreadsheet.global_push_failed";
    pub const SPREADSHEET_GLOBAL_LOADING: &str = "spreadsheet.global_loading";
    pub const SPREADSHEET_GLOBAL_LOAD_SUCCESS: &str = "spreadsheet.global_load_success";
    pub const SPREADSHEET_GLOBAL_LOAD_FAILED: &str = "spreadsheet.global_load_failed";

    // Kosenjo messages
    pub const KOSENJO_BEFORE_3_DAYS: &str = "kosenjo.before_3_days";
    pub const KOSENJO_BEFORE_1_DAY: &str = "kosenjo.before_1_day";
    pub const KOSENJO_QUALIFYING_START: &str = "kosenjo.qualifying_start";
    pub const KOSENJO_QUALIFYING_END: &str = "kosenjo.qualifying_end";
    pub const KOSENJO_QUALIFYING_END_NO_INTERVAL: &str = "kosenjo.qualifying_end_no_interval";
    pub const KOSENJO_MAIN_TOURNAMENT_BEFORE_1_DAY: &str = "kosenjo.main_tournament_before_1_day";
    pub const KOSENJO_MAIN_TOURNAMENT_DAY_START: &str = "kosenjo.main_tournament_day_start";
    pub const KOSENJO_MAIN_TOURNAMENT_HALF_DAY: &str = "kosenjo.main_tournament_half_day";
    pub const KOSENJO_MAIN_TOURNAMENT_DAY_END: &str = "kosenjo.main_tournament_day_end";
    pub const KOSENJO_MAIN_TOURNAMENT_END: &str = "kosenjo.main_tournament_end";
    pub const KOSENJO_SP_BATTLE_END: &str = "kosenjo.sp_battle_end";
    pub const KOSENJO_TEAM_ABILITY_1: &str = "kosenjo.team_ability_1";
    pub const KOSENJO_TEAM_ABILITY_2: &str = "kosenjo.team_ability_2";

    // Dorebara messages
    pub const DOREBARA_START: &str = "dorebara.start";
    pub const DOREBARA_END: &str = "dorebara.end";
    pub const DOREBARA_RESET: &str = "dorebara.reset";
    pub const DOREBARA_VARIANT: &str = "dorebara.variant";
    pub const DOREBARA_LAST_DAY: &str = "dorebara.last_day";

    // Bot messages
    pub const BOT_MENTION: &str = "bot.mention";
    pub const BOT_MENTION_SIX: &str = "bot.mention_six";
    pub const BOT_MENTION_CALLING: &str = "bot.mention_calling";

    // Omikuji messages
    pub const OMIKUJI_HIHI: &str = "omikuji.hihi";
    pub const OMIKUJI_HAKYOKU: &str = "omikuji.hakyoku";
    pub const OMIKUJI_OMEGA_UNIT: &str = "omikuji.omega_unit";

    // Recruitment display messages
    pub const RECRUITMENT_DISPLAY_NORMAL: &str = "recruitment.display.normal";
    pub const RECRUITMENT_DISPLAY_SIX_ELEMENTS: &str = "recruitment.display.six_elements";
    pub const RECRUITMENT_DISPLAY_EVENT_DATE_LABEL: &str = "recruitment.display.event_date_label";
    pub const RECRUITMENT_DISPLAY_DATE_FORMAT: &str = "recruitment.display.date_format";
    pub const RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL: &str =
        "recruitment.display.dismissal_times_label";
    pub const RECRUITMENT_DISPLAY_ELEMENT_FIRE: &str = "recruitment.display.element_fire";
    pub const RECRUITMENT_DISPLAY_ELEMENT_WATER: &str = "recruitment.display.element_water";
    pub const RECRUITMENT_DISPLAY_ELEMENT_EARTH: &str = "recruitment.display.element_earth";
    pub const RECRUITMENT_DISPLAY_ELEMENT_WIND: &str = "recruitment.display.element_wind";
    pub const RECRUITMENT_DISPLAY_ELEMENT_LIGHT: &str = "recruitment.display.element_light";
    pub const RECRUITMENT_DISPLAY_ELEMENT_DARK: &str = "recruitment.display.element_dark";
    pub const RECRUITMENT_DISPLAY_ALL_ELEMENTS: &str = "recruitment.display.all_elements";
    pub const RECRUITMENT_DISPLAY_NO_PARTICIPANTS: &str = "recruitment.display.no_participants";
    pub const RECRUITMENT_DISPLAY_LEAVE_ALL_BUTTON: &str = "recruitment.display.leave_all_button";

    // Recruitment notification messages
    pub const RECRUITMENT_NOTIFICATION_MEMBER_FULL: &str = "recruitment.notification.member_full";
    pub const RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES: &str =
        "recruitment.notification.before_5_minutes";
    pub const RECRUITMENT_NOTIFICATION_START: &str = "recruitment.notification.start";
    pub const RECRUITMENT_NOTIFICATION_DISMISSAL: &str = "recruitment.notification.dismissal";
    pub const RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS: &str =
        "recruitment.notification.dismissal_with_participants";

    // Timezone messages (deprecated - use GuildSettings instead)
    pub const TIMEZONE_SET_SUCCESS: &str = "timezone.set_success";
    pub const TIMEZONE_SHOW_CURRENT: &str = "timezone.show_current";

    // Guild settings messages
    pub const GUILD_SETTINGS_SET_SUCCESS: &str = "guild_settings.set_success";
    pub const GUILD_SETTINGS_SHOW_SUCCESS: &str = "guild_settings.show_success";
    pub const GUILD_SETTINGS_NOT_SET: &str = "guild_settings.not_set";

    // Recruitment role messages
    pub const RECRUITMENT_ROLE_ADD_SUCCESS: &str = "recruitment.role.add_success";
    pub const RECRUITMENT_ROLE_REMOVE_SUCCESS: &str = "recruitment.role.remove_success";

    // Recruitment command messages
    pub const RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED: &str =
        "recruitment.command.cancel_already_cancelled";
    pub const RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED: &str =
        "recruitment.command.cancel_message_deleted";
    pub const RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE: &str =
        "recruitment.command.cancel_invalid_message";
    pub const RECRUITMENT_COMMAND_CANCEL_NOT_FOUND: &str = "recruitment.command.cancel_not_found";
    pub const RECRUITMENT_COMMAND_CANCEL_EVENT_DATE_PASSED: &str =
        "recruitment.command.cancel_event_date_passed";
    pub const RECRUITMENT_COMMAND_CANCEL_ERROR: &str = "recruitment.command.cancel_error";
    pub const RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX: &str =
        "recruitment.command.cancelled_message_suffix";
    pub const RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS: &str =
        "recruitment.command.cancel_notification_no_participants";
    pub const RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS: &str =
        "recruitment.command.cancel_notification_with_participants";
    pub const RECRUITMENT_COMMAND_CANCELLING_PROGRESS: &str =
        "recruitment.command.cancelling_progress";
    pub const RECRUITMENT_COMMAND_CHANGE_NO_CHANGES: &str = "recruitment.command.change_no_changes";
    pub const RECRUITMENT_COMMAND_CHANGE_SUCCESS: &str = "recruitment.command.change_success";
    pub const RECRUITMENT_COMMAND_CANCEL_CONFIRM_PROMPT: &str =
        "recruitment.command.cancel_confirm_prompt";
    pub const RECRUITMENT_COMMAND_CANCEL_ABORTED: &str = "recruitment.command.cancel_aborted";
    pub const RECRUITMENT_COMMAND_CANCEL_UNKNOWN_SELECTION: &str =
        "recruitment.command.cancel_unknown_selection";
    pub const RECRUITMENT_COMMAND_CANCEL_TIMEOUT: &str = "recruitment.command.cancel_timeout";
    pub const RECRUITMENT_COMMAND_CANCEL_PERMISSION_DENIED: &str =
        "recruitment.command.cancel_permission_denied";
    pub const RECRUITMENT_COMMAND_CHANGE_PERMISSION_DENIED: &str =
        "recruitment.command.change_permission_denied";

    // General messages
    pub const MESSAGES_WELCOME: &str = "messages.welcome";
    pub const MESSAGES_HELP: &str = "messages.help";
    pub const MESSAGES_INIT_GUIDE: &str = "messages.init_guide";

    // Auto recruitment messages
    pub const AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED: &str =
        "auto_recruitment.channel_create_failed";
    pub const AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER: &str =
        "auto_recruitment.time_select_placeholder";
    pub const AUTO_RECRUITMENT_UNREGISTER_IN_CATEGORY_ERROR: &str =
        "auto_recruitment.unregister_in_category_error";
    pub const AUTO_RECRUITMENT_CATEGORY_SETUP_TIME_SELECT_MESSAGE: &str =
        "auto_recruitment.category_setup.time_select_message";
    pub const AUTO_RECRUITMENT_CATEGORY_SETUP_MATCHING_CHANNEL_MESSAGE: &str =
        "auto_recruitment.category_setup.matching_channel_message";
    pub const AUTO_RECRUITMENT_CATEGORY_SETUP_QUEST_CHANNEL_EMPTY_MESSAGE: &str =
        "auto_recruitment.category_setup.quest_channel_empty_message";
    pub const AUTO_RECRUITMENT_CATEGORY_SETUP_SELECTION_CHECK_BUTTON: &str =
        "auto_recruitment.category_setup.selection_check_button";
    pub const AUTO_RECRUITMENT_CATEGORY_SETUP_SELECTION_CHECK_MESSAGE: &str =
        "auto_recruitment.category_setup.selection_check_message";

    // App error messages
    pub const APP_ERROR_DATABASE: &str = "app_error.database";
    pub const APP_ERROR_DISCORD: &str = "app_error.discord";
    pub const APP_ERROR_CONFIG: &str = "app_error.config";
    pub const APP_ERROR_VALIDATION: &str = "app_error.validation";
    pub const APP_ERROR_DISCORD_OPERATION: &str = "app_error.discord_operation";
    pub const APP_ERROR_CHANNEL_CREATION_FAILED: &str = "app_error.channel_creation_failed";
    pub const APP_ERROR_IN_CATEGORY_CHANNEL: &str = "app_error.in_category_channel";

    // Schedule command messages
    pub const SCHEDULE_COMMAND_GENERATE_LOADING: &str = "schedule.command.generate.loading";
    pub const SCHEDULE_COMMAND_GENERATE_SUCCESS_TITLE: &str =
        "schedule.command.generate.success_title";
    pub const SCHEDULE_COMMAND_GENERATE_SUCCESS_DESCRIPTION: &str =
        "schedule.command.generate.success_description";
    pub const SCHEDULE_COMMAND_SHARED_SUCCESS_FIELD_NAME: &str =
        "schedule.command.shared.success_field_name";
    pub const SCHEDULE_COMMAND_GENERATE_SUCCESS_FIELD_VALUE: &str =
        "schedule.command.generate.success_field_value";
    pub const SCHEDULE_COMMAND_SHARED_SUCCESS_FOOTER: &str =
        "schedule.command.shared.success_footer";
    pub const SCHEDULE_COMMAND_GENERATE_ERROR_TITLE: &str = "schedule.command.generate.error_title";
    pub const SCHEDULE_COMMAND_GENERATE_ERROR_DESCRIPTION: &str =
        "schedule.command.generate.error_description";
    pub const SCHEDULE_COMMAND_SHARED_ERROR_FOOTER: &str = "schedule.command.shared.error_footer";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_LOADING: &str =
        "schedule.command.global_generate.loading";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_TITLE: &str =
        "schedule.command.global_generate.success_title";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_DESCRIPTION: &str =
        "schedule.command.global_generate.success_description";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_FIELD_VALUE: &str =
        "schedule.command.global_generate.success_field_value";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_TITLE: &str =
        "schedule.command.global_generate.error_title";
    pub const SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_DESCRIPTION: &str =
        "schedule.command.global_generate.error_description";
    pub const SCHEDULE_COMMAND_LIST_TITLE: &str = "schedule.command.list.title";
    pub const SCHEDULE_COMMAND_LIST_EMPTY_DESCRIPTION: &str =
        "schedule.command.list.empty_description";
    pub const SCHEDULE_COMMAND_LIST_FOOTER: &str = "schedule.command.list.footer";
    pub const SCHEDULE_COMMAND_HISTORY_TITLE: &str = "schedule.command.history.title";
    pub const SCHEDULE_COMMAND_HISTORY_EMPTY_DESCRIPTION: &str =
        "schedule.command.history.empty_description";
    pub const SCHEDULE_COMMAND_HISTORY_TITLE_WITH_DAYS: &str =
        "schedule.command.history.title_with_days";
    pub const SCHEDULE_COMMAND_HISTORY_FOOTER: &str = "schedule.command.history.footer";
    pub const SCHEDULE_COMMAND_STATS_TITLE_WITH_DAYS: &str =
        "schedule.command.stats.title_with_days";
    pub const SCHEDULE_COMMAND_STATS_FOOTER: &str = "schedule.command.stats.footer";
    pub const SCHEDULE_COMMAND_STATS_DESCRIPTION_HEADER: &str =
        "schedule.command.stats.description_header";
    pub const SCHEDULE_COMMAND_STATS_MESSAGE_TYPE_HEADER: &str =
        "schedule.command.stats.message_type_header";
    pub const SCHEDULE_COMMAND_STATS_OTHER_TYPES: &str = "schedule.command.stats.other_types";

    // Recruitment schedule command messages
    pub const RECRUITMENT_SCHEDULE_LIST_EMPTY_ALL: &str = "recruitment.schedule.list.empty_all";
    pub const RECRUITMENT_SCHEDULE_LIST_EMPTY_SELF: &str = "recruitment.schedule.list.empty_self";
    pub const RECRUITMENT_SCHEDULE_LIST_TITLE_ALL: &str = "recruitment.schedule.list.title_all";
    pub const RECRUITMENT_SCHEDULE_LIST_TITLE_SELF: &str = "recruitment.schedule.list.title_self";
    pub const RECRUITMENT_SCHEDULE_LIST_STATUS_ENABLED: &str =
        "recruitment.schedule.list.status_enabled";
    pub const RECRUITMENT_SCHEDULE_LIST_STATUS_DISABLED: &str =
        "recruitment.schedule.list.status_disabled";
    pub const RECRUITMENT_SCHEDULE_LIST_DISMISSAL_PREFIX: &str =
        "recruitment.schedule.list.dismissal_prefix";
    pub const RECRUITMENT_SCHEDULE_LIST_MORE_COUNT: &str = "recruitment.schedule.list.more_count";
    pub const RECRUITMENT_SCHEDULE_LIST_FOOTER: &str = "recruitment.schedule.list.footer";

    // Recruitment list command messages
    pub const RECRUITMENT_LIST_TITLE: &str = "recruitment.list.title";
    pub const RECRUITMENT_LIST_EMPTY: &str = "recruitment.list.empty";
    pub const RECRUITMENT_LIST_LINK_TEXT: &str = "recruitment.list.link_text";
    pub const RECRUITMENT_LIST_MORE_COUNT: &str = "recruitment.list.more_count";
    pub const RECRUITMENT_LIST_FOOTER: &str = "recruitment.list.footer";

    // Quest list command messages
    pub const QUEST_LIST_TITLE_ALL: &str = "quest.list.title_all";
    pub const QUEST_LIST_TITLE_ENABLED: &str = "quest.list.title_enabled";
    pub const QUEST_LIST_TITLE_DISABLED: &str = "quest.list.title_disabled";
    pub const QUEST_LIST_MORE_COUNT: &str = "quest.list.more_count";
    pub const QUEST_LIST_EMPTY_ENABLED: &str = "quest.list.empty_enabled";
    pub const QUEST_LIST_EMPTY_DISABLED: &str = "quest.list.empty_disabled";

    pub const RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_TITLE: &str =
        "recruitment.schedule.toggle.success_title";
    pub const RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_DESCRIPTION: &str =
        "recruitment.schedule.toggle.success_description";
    pub const RECRUITMENT_SCHEDULE_DELETE_SUCCESS_TITLE: &str =
        "recruitment.schedule.delete.success_title";
    pub const RECRUITMENT_SCHEDULE_DELETE_SUCCESS_DESCRIPTION: &str =
        "recruitment.schedule.delete.success_description";

    // Recruitment role show messages
    pub const RECRUITMENT_ROLE_SHOW_NOT_REGISTERED: &str = "recruitment.role.show.not_registered";
    pub const RECRUITMENT_ROLE_SHOW_HEADER: &str = "recruitment.role.show.header";
    pub const RECRUITMENT_ROLE_SHOW_SECTION_ALL: &str = "recruitment.role.show.section_all";
    pub const RECRUITMENT_ROLE_SHOW_SECTION_QUEST: &str = "recruitment.role.show.section_quest";
    pub const RECRUITMENT_ROLE_SHOW_UNKNOWN_QUEST: &str = "recruitment.role.show.unknown_quest";

    // Recruitment change panel messages
    pub const RECRUITMENT_COMMAND_CHANGE_PANEL_UNCHANGED: &str =
        "recruitment.command.change.panel_unchanged";
    pub const RECRUITMENT_COMMAND_CHANGE_PANEL_CONTENT: &str =
        "recruitment.command.change.panel_content";
    pub const RECRUITMENT_COMMAND_CHANGE_OPTION_QUEST_UNCHANGED: &str =
        "recruitment.command.change.option_quest_unchanged";
    pub const RECRUITMENT_COMMAND_CHANGE_OPTION_STYLE_UNCHANGED: &str =
        "recruitment.command.change.option_style_unchanged";
    pub const RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_QUEST: &str =
        "recruitment.command.change.placeholder_quest";
    pub const RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_STYLE: &str =
        "recruitment.command.change.placeholder_style";
    pub const RECRUITMENT_COMMAND_CHANGE_BUTTON_OPEN_DATE: &str =
        "recruitment.command.change.button_open_date";
    pub const RECRUITMENT_COMMAND_CHANGE_BUTTON_CLEAR_DATE: &str =
        "recruitment.command.change.button_clear_date";
    pub const RECRUITMENT_COMMAND_CHANGE_BUTTON_APPLY: &str =
        "recruitment.command.change.button_apply";
    pub const RECRUITMENT_COMMAND_CHANGE_MODAL_TITLE: &str =
        "recruitment.command.change.modal_title";
    pub const RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_LABEL: &str =
        "recruitment.command.change.modal_event_date_label";
    pub const RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_PLACEHOLDER: &str =
        "recruitment.command.change.modal_event_date_placeholder";
    pub const RECRUITMENT_COMMAND_CHANGE_MODAL_ABSOLUTE_DATETIME_REQUIRED: &str =
        "recruitment.command.change.modal.absolute_datetime_required";
    pub const RECRUITMENT_COMMAND_CHANGE_MODAL_PARSE_FAILED: &str =
        "recruitment.command.change.modal.parse_failed";

    // Channel show messages
    pub const CHANNEL_SHOW_EMPTY: &str = "channel.show.empty";
    pub const CHANNEL_SHOW_HEADER: &str = "channel.show.header";
    pub const CHANNEL_SHOW_UNSET: &str = "channel.show.unset";

    // Auto recruitment interaction messages
    pub const AUTO_RECRUITMENT_QUEST_SELECT_REQUIRED: &str =
        "auto_recruitment.quest_select_required";
    pub const AUTO_RECRUITMENT_QUEST_SELECT_REGISTERED: &str =
        "auto_recruitment.quest_select_registered";
    pub const AUTO_RECRUITMENT_TIME_SELECT_REQUIRED: &str = "auto_recruitment.time_select_required";
    pub const AUTO_RECRUITMENT_TIME_SELECT_REGISTERED: &str =
        "auto_recruitment.time_select_registered";
    pub const AUTO_RECRUITMENT_STATUS_HEADER: &str = "auto_recruitment.status.header";
    pub const AUTO_RECRUITMENT_STATUS_QUEST_EMPTY: &str = "auto_recruitment.status.quest_empty";
    pub const AUTO_RECRUITMENT_STATUS_QUEST_COUNT: &str = "auto_recruitment.status.quest_count";
    pub const AUTO_RECRUITMENT_STATUS_QUEST_IDS: &str = "auto_recruitment.status.quest_ids";
    pub const AUTO_RECRUITMENT_STATUS_TIME_EMPTY: &str = "auto_recruitment.status.time_empty";
    pub const AUTO_RECRUITMENT_STATUS_TIME_HEADER: &str = "auto_recruitment.status.time_header";
    pub const AUTO_RECRUITMENT_STATUS_TIME_SLOT: &str = "auto_recruitment.status.time_slot";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_FIRE: &str =
        "auto_recruitment.presenter.element_fire";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_WATER: &str =
        "auto_recruitment.presenter.element_water";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_EARTH: &str =
        "auto_recruitment.presenter.element_earth";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_WIND: &str =
        "auto_recruitment.presenter.element_wind";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_LIGHT: &str =
        "auto_recruitment.presenter.element_light";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_DARK: &str =
        "auto_recruitment.presenter.element_dark";
    pub const AUTO_RECRUITMENT_PRESENTER_JOIN_BUTTON: &str =
        "auto_recruitment.presenter.join_button";
    pub const AUTO_RECRUITMENT_PRESENTER_ELEMENT_PLACEHOLDER: &str =
        "auto_recruitment.presenter.element_placeholder";
    pub const AUTO_RECRUITMENT_PRESENTER_QUEST_SELECT_PLACEHOLDER: &str =
        "auto_recruitment.presenter.quest_select_placeholder";
    pub const AUTO_RECRUITMENT_PRESENTER_QUEST_SELECT_MESSAGE: &str =
        "auto_recruitment.presenter.quest_select_message";
    pub const AUTO_RECRUITMENT_PRESENTER_TIME_SELECT_PLACEHOLDER: &str =
        "auto_recruitment.presenter.time_select_placeholder";
    pub const AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_TITLE: &str =
        "auto_recruitment.presenter.setup_complete_title";
    pub const AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_DESCRIPTION: &str =
        "auto_recruitment.presenter.setup_complete_description";
    pub const AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_QUEST_FIELD: &str =
        "auto_recruitment.presenter.setup_complete_quest_field";
    pub const AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_TIME_FIELD: &str =
        "auto_recruitment.presenter.setup_complete_time_field";

    // Help embed messages
    pub const HELP_EMBED_TITLE: &str = "help.embed.title";
    pub const HELP_EMBED_DESCRIPTION: &str = "help.embed.description";
    pub const HELP_EMBED_COMMANDS_FIELD_TITLE: &str = "help.embed.commands_field_title";
    pub const HELP_EMBED_COMMANDS_FIELD_VALUE: &str = "help.embed.commands_field_value";
    pub const HELP_EMBED_RECRUIT_FIELD_VALUE: &str = "help.embed.recruit_field_value";
    pub const HELP_EMBED_ENVIRON_LOAD_FIELD_VALUE: &str = "help.embed.environ_load_field_value";
    pub const HELP_EMBED_GSPREAD_LOAD_FIELD_VALUE: &str = "help.embed.gspread_load_field_value";
    pub const HELP_EMBED_GSPREAD_PUSH_FIELD_VALUE: &str = "help.embed.gspread_push_field_value";
    pub const HELP_EMBED_HELP_FIELD_VALUE: &str = "help.embed.help_field_value";
    pub const HELP_EMBED_FOOTER: &str = "help.embed.footer";
}

/// メッセージID定義
///
/// メッセージIDを型安全に管理するためのenum
/// 全てのメッセージIDはこのenumで定義される
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageTextId {
    // Common messages
    CommonSuccess,
    CommonError,
    CommonWarning,
    CommonInfo,
    CommonYes,
    CommonNo,
    CommonCancel,
    CommonConfirm,
    CommonLoading,
    CommonUnknown,

    // Recruitment UI messages
    RecruitmentUiTitle,
    RecruitmentUiNewRecruitment,
    RecruitmentUiRecruitmentCancelled,
    RecruitmentUiRecruitmentClosed,
    RecruitmentUiRecruitmentFull,
    RecruitmentUiJoinSuccess,
    RecruitmentUiLeaveSuccess,
    RecruitmentUiNotFound,
    RecruitmentUiAlreadyJoined,
    RecruitmentUiNotJoined,

    // Error messages
    ErrorsInvalidInput,
    ErrorsPermissionDenied,
    ErrorsInternalError,
    ErrorsUserNotFound,
    ErrorsCommandFailed,
    ErrorsEnvVarNotSet,
    ErrorsGuildOnly,
    ErrorsSpreadsheetNotRegistered,
    ErrorsSpreadsheetConfigFetchFailed,

    // Spreadsheet messages
    SpreadsheetLoading,
    SpreadsheetLoadSuccess,
    SpreadsheetLoadPartialSuccess,
    SpreadsheetLoadFailed,
    SpreadsheetRegistering,
    SpreadsheetRegisterSuccess,
    SpreadsheetRegisterFailed,
    SpreadsheetPushing,
    SpreadsheetPushSuccess,
    SpreadsheetPushPartialSuccess,
    SpreadsheetPushFailed,
    SpreadsheetGlobalPushing,
    SpreadsheetGlobalPushSuccess,
    SpreadsheetGlobalPushPartialSuccess,
    SpreadsheetGlobalPushFailed,
    SpreadsheetGlobalLoading,
    SpreadsheetGlobalLoadSuccess,
    SpreadsheetGlobalLoadFailed,

    // Kosenjo messages
    KosenjoBefore3Days,
    KosenjoBefore1Day,
    KosenjoQualifyingStart,
    KosenjoQualifyingEnd,
    KosenjoQualifyingEndNoInterval,
    KosenjoMainTournamentBefore1Day,
    KosenjoMainTournamentDayStart,
    KosenjoMainTournamentHalfDay,
    KosenjoMainTournamentDayEnd,
    KosenjoMainTournamentEnd,
    KosenjoSpBattleEnd,
    KosenjoTeamAbility1,
    KosenjoTeamAbility2,

    // Dorebara messages
    DorebaraStart,
    DorebaraEnd,
    DorebaraReset,
    DorebaraVariant,
    DorebaraLastDay,

    // Bot messages
    BotMention,
    BotMentionSix,
    BotMentionCalling,

    // Omikuji messages
    OmikujiHihi,
    OmikujiHakyoku,
    OmikujiOmegaUnit,

    // Recruitment display messages
    RecruitmentDisplayNormal,
    RecruitmentDisplaySixElements,
    RecruitmentDisplayEventDateLabel,
    RecruitmentDisplayDateFormat,
    RecruitmentDisplayDismissalTimesLabel,
    RecruitmentDisplayElementFire,
    RecruitmentDisplayElementWater,
    RecruitmentDisplayElementEarth,
    RecruitmentDisplayElementWind,
    RecruitmentDisplayElementLight,
    RecruitmentDisplayElementDark,
    RecruitmentDisplayAllElements,
    RecruitmentDisplayNoParticipants,
    RecruitmentDisplayLeaveAllButton,

    // Recruitment notification messages
    RecruitmentNotificationMemberFull,
    RecruitmentNotificationBefore5Minutes,
    RecruitmentNotificationStart,
    RecruitmentNotificationDismissal,
    RecruitmentNotificationDismissalWithParticipants,

    // Timezone messages (deprecated - use GuildSettings instead)
    TimezoneSetSuccess,
    TimezoneShowCurrent,

    // Guild settings messages
    GuildSettingsSetSuccess,
    GuildSettingsShowSuccess,
    GuildSettingsNotSet,

    // Recruitment role messages
    RecruitmentRoleAddSuccess,
    RecruitmentRoleRemoveSuccess,

    // Recruitment command messages
    RecruitmentCommandCancelAlreadyCancelled,
    RecruitmentCommandCancelMessageDeleted,
    RecruitmentCommandCancelInvalidMessage,
    RecruitmentCommandCancelNotFound,
    RecruitmentCommandCancelEventDatePassed,
    RecruitmentCommandCancelError,
    RecruitmentCommandCancelledMessageSuffix,
    RecruitmentCommandCancelNotificationNoParticipants,
    RecruitmentCommandCancelNotificationWithParticipants,
    RecruitmentCommandCancellingProgress,
    RecruitmentCommandChangeNoChanges,
    RecruitmentCommandChangeSuccess,
    RecruitmentCommandCancelConfirmPrompt,
    RecruitmentCommandCancelAborted,
    RecruitmentCommandCancelUnknownSelection,
    RecruitmentCommandCancelTimeout,
    RecruitmentCommandCancelPermissionDenied,
    RecruitmentCommandChangePermissionDenied,

    // General messages
    MessagesWelcome,
    MessagesHelp,
    MessagesInitGuide,

    // Auto recruitment messages
    AutoRecruitmentChannelCreateFailed,
    AutoRecruitmentTimeSelectPlaceholder,
    AutoRecruitmentUnregisterInCategoryError,
    AutoRecruitmentCategorySetupTimeSelectMessage,
    AutoRecruitmentCategorySetupMatchingChannelMessage,
    AutoRecruitmentCategorySetupQuestChannelEmptyMessage,
    AutoRecruitmentCategorySetupSelectionCheckButton,
    AutoRecruitmentCategorySetupSelectionCheckMessage,

    // App error messages
    AppErrorDatabase,
    AppErrorDiscord,
    AppErrorConfig,
    AppErrorValidation,
    AppErrorDiscordOperation,
    AppErrorChannelCreationFailed,
    AppErrorInCategoryChannel,

    // Schedule command messages
    ScheduleCommandGenerateLoading,
    ScheduleCommandGenerateSuccessTitle,
    ScheduleCommandGenerateSuccessDescription,
    ScheduleCommandSharedSuccessFieldName,
    ScheduleCommandGenerateSuccessFieldValue,
    ScheduleCommandSharedSuccessFooter,
    ScheduleCommandGenerateErrorTitle,
    ScheduleCommandGenerateErrorDescription,
    ScheduleCommandSharedErrorFooter,
    ScheduleCommandGlobalGenerateLoading,
    ScheduleCommandGlobalGenerateSuccessTitle,
    ScheduleCommandGlobalGenerateSuccessDescription,
    ScheduleCommandGlobalGenerateSuccessFieldValue,
    ScheduleCommandGlobalGenerateErrorTitle,
    ScheduleCommandGlobalGenerateErrorDescription,
    ScheduleCommandListTitle,
    ScheduleCommandListEmptyDescription,
    ScheduleCommandListFooter,
    ScheduleCommandHistoryTitle,
    ScheduleCommandHistoryEmptyDescription,
    ScheduleCommandHistoryTitleWithDays,
    ScheduleCommandHistoryFooter,
    ScheduleCommandStatsTitleWithDays,
    ScheduleCommandStatsFooter,
    ScheduleCommandStatsDescriptionHeader,
    ScheduleCommandStatsMessageTypeHeader,
    ScheduleCommandStatsOtherTypes,

    // Recruitment schedule command messages
    RecruitmentScheduleListEmptyAll,
    RecruitmentScheduleListEmptySelf,
    RecruitmentScheduleListTitleAll,
    RecruitmentScheduleListTitleSelf,
    RecruitmentScheduleListStatusEnabled,
    RecruitmentScheduleListStatusDisabled,
    RecruitmentScheduleListDismissalPrefix,
    RecruitmentScheduleListMoreCount,
    RecruitmentScheduleListFooter,

    // 募集一覧コマンドメッセージ
    RecruitmentListTitle,
    RecruitmentListEmpty,
    RecruitmentListLinkText,
    RecruitmentListMoreCount,
    RecruitmentListFooter,

    // Quest list command messages
    QuestListTitleAll,
    QuestListTitleEnabled,
    QuestListTitleDisabled,
    QuestListMoreCount,
    QuestListEmptyEnabled,
    QuestListEmptyDisabled,

    RecruitmentScheduleToggleSuccessTitle,
    RecruitmentScheduleToggleSuccessDescription,
    RecruitmentScheduleDeleteSuccessTitle,
    RecruitmentScheduleDeleteSuccessDescription,

    // Recruitment role show messages
    RecruitmentRoleShowNotRegistered,
    RecruitmentRoleShowHeader,
    RecruitmentRoleShowSectionAll,
    RecruitmentRoleShowSectionQuest,
    RecruitmentRoleShowUnknownQuest,

    // Recruitment change panel messages
    RecruitmentCommandChangePanelUnchanged,
    RecruitmentCommandChangePanelContent,
    RecruitmentCommandChangeOptionQuestUnchanged,
    RecruitmentCommandChangeOptionStyleUnchanged,
    RecruitmentCommandChangePlaceholderQuest,
    RecruitmentCommandChangePlaceholderStyle,
    RecruitmentCommandChangeButtonOpenDate,
    RecruitmentCommandChangeButtonClearDate,
    RecruitmentCommandChangeButtonApply,
    RecruitmentCommandChangeModalTitle,
    RecruitmentCommandChangeModalEventDateLabel,
    RecruitmentCommandChangeModalEventDatePlaceholder,
    CommonErrorPrefix,
    RecruitmentCommandChangeModalAbsoluteDatetimeRequired,
    RecruitmentCommandChangeModalParseFailed,

    // Channel show messages
    ChannelShowEmpty,
    ChannelShowHeader,
    ChannelShowUnset,

    // Auto recruitment interaction messages
    AutoRecruitmentQuestSelectRequired,
    AutoRecruitmentQuestSelectRegistered,
    AutoRecruitmentTimeSelectRequired,
    AutoRecruitmentTimeSelectRegistered,
    AutoRecruitmentStatusHeader,
    AutoRecruitmentStatusQuestEmpty,
    AutoRecruitmentStatusQuestCount,
    AutoRecruitmentStatusQuestIds,
    AutoRecruitmentStatusTimeEmpty,
    AutoRecruitmentStatusTimeHeader,
    AutoRecruitmentStatusTimeSlot,
    AutoRecruitmentPresenterElementFire,
    AutoRecruitmentPresenterElementWater,
    AutoRecruitmentPresenterElementEarth,
    AutoRecruitmentPresenterElementWind,
    AutoRecruitmentPresenterElementLight,
    AutoRecruitmentPresenterElementDark,
    AutoRecruitmentPresenterJoinButton,
    AutoRecruitmentPresenterElementPlaceholder,
    AutoRecruitmentPresenterQuestSelectPlaceholder,
    AutoRecruitmentPresenterQuestSelectMessage,
    AutoRecruitmentPresenterTimeSelectPlaceholder,
    AutoRecruitmentPresenterSetupCompleteTitle,
    AutoRecruitmentPresenterSetupCompleteDescription,
    AutoRecruitmentPresenterSetupCompleteQuestField,
    AutoRecruitmentPresenterSetupCompleteTimeField,

    // Help embed messages
    HelpEmbedTitle,
    HelpEmbedDescription,
    HelpEmbedCommandsFieldTitle,
    HelpEmbedCommandsFieldValue,
    HelpEmbedRecruitFieldValue,
    HelpEmbedEnvironLoadFieldValue,
    HelpEmbedGspreadLoadFieldValue,
    HelpEmbedGspreadPushFieldValue,
    HelpEmbedHelpFieldValue,
    HelpEmbedFooter,
}

impl MessageTextId {
    /// MessageIdを文字列キーに変換
    ///
    /// # 戻り値
    /// YAMLファイル内のキー（例: "timezone.show_current"）
    ///
    /// # 実装ノート
    /// 各キーの定義は `keys` モジュールで一元管理されている。
    /// これにより、`as_str()` と `yaml_loader.rs` で同じ定数を参照でき、
    /// 単一の情報源(Single Source of Truth)を実現している。
    pub fn as_str(&self) -> &'static str {
        match self {
            // Common
            MessageTextId::CommonSuccess => keys::COMMON_SUCCESS,
            MessageTextId::CommonError => keys::COMMON_ERROR,
            MessageTextId::CommonErrorPrefix => keys::COMMON_ERROR_PREFIX,
            MessageTextId::CommonWarning => keys::COMMON_WARNING,
            MessageTextId::CommonInfo => keys::COMMON_INFO,
            MessageTextId::CommonYes => keys::COMMON_YES,
            MessageTextId::CommonNo => keys::COMMON_NO,
            MessageTextId::CommonCancel => keys::COMMON_CANCEL,
            MessageTextId::CommonConfirm => keys::COMMON_CONFIRM,
            MessageTextId::CommonLoading => keys::COMMON_LOADING,
            MessageTextId::CommonUnknown => keys::COMMON_UNKNOWN,

            // Recruitment UI
            MessageTextId::RecruitmentUiTitle => keys::RECRUITMENT_UI_TITLE,
            MessageTextId::RecruitmentUiNewRecruitment => keys::RECRUITMENT_UI_NEW_RECRUITMENT,
            MessageTextId::RecruitmentUiRecruitmentCancelled => {
                keys::RECRUITMENT_UI_RECRUITMENT_CANCELLED
            }
            MessageTextId::RecruitmentUiRecruitmentClosed => {
                keys::RECRUITMENT_UI_RECRUITMENT_CLOSED
            }
            MessageTextId::RecruitmentUiRecruitmentFull => keys::RECRUITMENT_UI_RECRUITMENT_FULL,
            MessageTextId::RecruitmentUiJoinSuccess => keys::RECRUITMENT_UI_JOIN_SUCCESS,
            MessageTextId::RecruitmentUiLeaveSuccess => keys::RECRUITMENT_UI_LEAVE_SUCCESS,
            MessageTextId::RecruitmentUiNotFound => keys::RECRUITMENT_UI_NOT_FOUND,
            MessageTextId::RecruitmentUiAlreadyJoined => keys::RECRUITMENT_UI_ALREADY_JOINED,
            MessageTextId::RecruitmentUiNotJoined => keys::RECRUITMENT_UI_NOT_JOINED,

            // Errors
            MessageTextId::ErrorsInvalidInput => keys::ERRORS_INVALID_INPUT,
            MessageTextId::ErrorsPermissionDenied => keys::ERRORS_PERMISSION_DENIED,
            MessageTextId::ErrorsInternalError => keys::ERRORS_INTERNAL_ERROR,
            MessageTextId::ErrorsUserNotFound => keys::ERRORS_USER_NOT_FOUND,
            MessageTextId::ErrorsCommandFailed => keys::ERRORS_COMMAND_FAILED,
            MessageTextId::ErrorsEnvVarNotSet => keys::ERRORS_ENV_VAR_NOT_SET,
            MessageTextId::ErrorsGuildOnly => keys::ERRORS_GUILD_ONLY,
            MessageTextId::ErrorsSpreadsheetNotRegistered => {
                keys::ERRORS_SPREADSHEET_NOT_REGISTERED
            }
            MessageTextId::ErrorsSpreadsheetConfigFetchFailed => {
                keys::ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED
            }

            // Spreadsheet
            MessageTextId::SpreadsheetLoading => keys::SPREADSHEET_LOADING,
            MessageTextId::SpreadsheetLoadSuccess => keys::SPREADSHEET_LOAD_SUCCESS,
            MessageTextId::SpreadsheetLoadPartialSuccess => keys::SPREADSHEET_LOAD_PARTIAL_SUCCESS,
            MessageTextId::SpreadsheetLoadFailed => keys::SPREADSHEET_LOAD_FAILED,
            MessageTextId::SpreadsheetRegistering => keys::SPREADSHEET_REGISTERING,
            MessageTextId::SpreadsheetRegisterSuccess => keys::SPREADSHEET_REGISTER_SUCCESS,
            MessageTextId::SpreadsheetRegisterFailed => keys::SPREADSHEET_REGISTER_FAILED,
            MessageTextId::SpreadsheetPushing => keys::SPREADSHEET_PUSHING,
            MessageTextId::SpreadsheetPushSuccess => keys::SPREADSHEET_PUSH_SUCCESS,
            MessageTextId::SpreadsheetPushPartialSuccess => keys::SPREADSHEET_PUSH_PARTIAL_SUCCESS,
            MessageTextId::SpreadsheetPushFailed => keys::SPREADSHEET_PUSH_FAILED,
            MessageTextId::SpreadsheetGlobalPushing => keys::SPREADSHEET_GLOBAL_PUSHING,
            MessageTextId::SpreadsheetGlobalPushSuccess => keys::SPREADSHEET_GLOBAL_PUSH_SUCCESS,
            MessageTextId::SpreadsheetGlobalPushPartialSuccess => {
                keys::SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS
            }
            MessageTextId::SpreadsheetGlobalPushFailed => keys::SPREADSHEET_GLOBAL_PUSH_FAILED,
            MessageTextId::SpreadsheetGlobalLoading => keys::SPREADSHEET_GLOBAL_LOADING,
            MessageTextId::SpreadsheetGlobalLoadSuccess => keys::SPREADSHEET_GLOBAL_LOAD_SUCCESS,
            MessageTextId::SpreadsheetGlobalLoadFailed => keys::SPREADSHEET_GLOBAL_LOAD_FAILED,

            // Kosenjo
            MessageTextId::KosenjoBefore3Days => keys::KOSENJO_BEFORE_3_DAYS,
            MessageTextId::KosenjoBefore1Day => keys::KOSENJO_BEFORE_1_DAY,
            MessageTextId::KosenjoQualifyingStart => keys::KOSENJO_QUALIFYING_START,
            MessageTextId::KosenjoQualifyingEnd => keys::KOSENJO_QUALIFYING_END,
            MessageTextId::KosenjoQualifyingEndNoInterval => {
                keys::KOSENJO_QUALIFYING_END_NO_INTERVAL
            }
            MessageTextId::KosenjoMainTournamentBefore1Day => {
                keys::KOSENJO_MAIN_TOURNAMENT_BEFORE_1_DAY
            }
            MessageTextId::KosenjoMainTournamentDayStart => keys::KOSENJO_MAIN_TOURNAMENT_DAY_START,
            MessageTextId::KosenjoMainTournamentHalfDay => keys::KOSENJO_MAIN_TOURNAMENT_HALF_DAY,
            MessageTextId::KosenjoMainTournamentDayEnd => keys::KOSENJO_MAIN_TOURNAMENT_DAY_END,
            MessageTextId::KosenjoMainTournamentEnd => keys::KOSENJO_MAIN_TOURNAMENT_END,
            MessageTextId::KosenjoSpBattleEnd => keys::KOSENJO_SP_BATTLE_END,
            MessageTextId::KosenjoTeamAbility1 => keys::KOSENJO_TEAM_ABILITY_1,
            MessageTextId::KosenjoTeamAbility2 => keys::KOSENJO_TEAM_ABILITY_2,

            // Dorebara
            MessageTextId::DorebaraStart => keys::DOREBARA_START,
            MessageTextId::DorebaraEnd => keys::DOREBARA_END,
            MessageTextId::DorebaraReset => keys::DOREBARA_RESET,
            MessageTextId::DorebaraVariant => keys::DOREBARA_VARIANT,
            MessageTextId::DorebaraLastDay => keys::DOREBARA_LAST_DAY,

            // Bot
            MessageTextId::BotMention => keys::BOT_MENTION,
            MessageTextId::BotMentionSix => keys::BOT_MENTION_SIX,
            MessageTextId::BotMentionCalling => keys::BOT_MENTION_CALLING,

            // Omikuji
            MessageTextId::OmikujiHihi => keys::OMIKUJI_HIHI,
            MessageTextId::OmikujiHakyoku => keys::OMIKUJI_HAKYOKU,
            MessageTextId::OmikujiOmegaUnit => keys::OMIKUJI_OMEGA_UNIT,

            // Recruitment display
            MessageTextId::RecruitmentDisplayNormal => keys::RECRUITMENT_DISPLAY_NORMAL,
            MessageTextId::RecruitmentDisplaySixElements => keys::RECRUITMENT_DISPLAY_SIX_ELEMENTS,
            MessageTextId::RecruitmentDisplayEventDateLabel => {
                keys::RECRUITMENT_DISPLAY_EVENT_DATE_LABEL
            }
            MessageTextId::RecruitmentDisplayDateFormat => keys::RECRUITMENT_DISPLAY_DATE_FORMAT,
            MessageTextId::RecruitmentDisplayDismissalTimesLabel => {
                keys::RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL
            }
            MessageTextId::RecruitmentDisplayElementFire => keys::RECRUITMENT_DISPLAY_ELEMENT_FIRE,
            MessageTextId::RecruitmentDisplayElementWater => {
                keys::RECRUITMENT_DISPLAY_ELEMENT_WATER
            }
            MessageTextId::RecruitmentDisplayElementEarth => {
                keys::RECRUITMENT_DISPLAY_ELEMENT_EARTH
            }
            MessageTextId::RecruitmentDisplayElementWind => keys::RECRUITMENT_DISPLAY_ELEMENT_WIND,
            MessageTextId::RecruitmentDisplayElementLight => {
                keys::RECRUITMENT_DISPLAY_ELEMENT_LIGHT
            }
            MessageTextId::RecruitmentDisplayElementDark => keys::RECRUITMENT_DISPLAY_ELEMENT_DARK,
            MessageTextId::RecruitmentDisplayAllElements => keys::RECRUITMENT_DISPLAY_ALL_ELEMENTS,
            MessageTextId::RecruitmentDisplayNoParticipants => {
                keys::RECRUITMENT_DISPLAY_NO_PARTICIPANTS
            }
            MessageTextId::RecruitmentDisplayLeaveAllButton => {
                keys::RECRUITMENT_DISPLAY_LEAVE_ALL_BUTTON
            }

            // Recruitment notification
            MessageTextId::RecruitmentNotificationMemberFull => {
                keys::RECRUITMENT_NOTIFICATION_MEMBER_FULL
            }
            MessageTextId::RecruitmentNotificationBefore5Minutes => {
                keys::RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES
            }
            MessageTextId::RecruitmentNotificationStart => keys::RECRUITMENT_NOTIFICATION_START,
            MessageTextId::RecruitmentNotificationDismissal => {
                keys::RECRUITMENT_NOTIFICATION_DISMISSAL
            }
            MessageTextId::RecruitmentNotificationDismissalWithParticipants => {
                keys::RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS
            }

            // Timezone
            MessageTextId::TimezoneSetSuccess => keys::TIMEZONE_SET_SUCCESS,
            MessageTextId::TimezoneShowCurrent => keys::TIMEZONE_SHOW_CURRENT,

            // Guild settings
            MessageTextId::GuildSettingsSetSuccess => keys::GUILD_SETTINGS_SET_SUCCESS,
            MessageTextId::GuildSettingsShowSuccess => keys::GUILD_SETTINGS_SHOW_SUCCESS,
            MessageTextId::GuildSettingsNotSet => keys::GUILD_SETTINGS_NOT_SET,

            // Recruitment role
            MessageTextId::RecruitmentRoleAddSuccess => keys::RECRUITMENT_ROLE_ADD_SUCCESS,
            MessageTextId::RecruitmentRoleRemoveSuccess => keys::RECRUITMENT_ROLE_REMOVE_SUCCESS,

            // Recruitment command
            MessageTextId::RecruitmentCommandCancelAlreadyCancelled => {
                keys::RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED
            }
            MessageTextId::RecruitmentCommandCancelMessageDeleted => {
                keys::RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED
            }
            MessageTextId::RecruitmentCommandCancelInvalidMessage => {
                keys::RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE
            }
            MessageTextId::RecruitmentCommandCancelNotFound => {
                keys::RECRUITMENT_COMMAND_CANCEL_NOT_FOUND
            }
            MessageTextId::RecruitmentCommandCancelEventDatePassed => {
                keys::RECRUITMENT_COMMAND_CANCEL_EVENT_DATE_PASSED
            }
            MessageTextId::RecruitmentCommandCancelError => keys::RECRUITMENT_COMMAND_CANCEL_ERROR,
            MessageTextId::RecruitmentCommandCancelledMessageSuffix => {
                keys::RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX
            }
            MessageTextId::RecruitmentCommandCancelNotificationNoParticipants => {
                keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS
            }
            MessageTextId::RecruitmentCommandCancelNotificationWithParticipants => {
                keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS
            }
            MessageTextId::RecruitmentCommandCancellingProgress => {
                keys::RECRUITMENT_COMMAND_CANCELLING_PROGRESS
            }
            MessageTextId::RecruitmentCommandChangeNoChanges => {
                keys::RECRUITMENT_COMMAND_CHANGE_NO_CHANGES
            }
            MessageTextId::RecruitmentCommandChangeSuccess => {
                keys::RECRUITMENT_COMMAND_CHANGE_SUCCESS
            }
            MessageTextId::RecruitmentCommandCancelConfirmPrompt => {
                keys::RECRUITMENT_COMMAND_CANCEL_CONFIRM_PROMPT
            }
            MessageTextId::RecruitmentCommandCancelAborted => {
                keys::RECRUITMENT_COMMAND_CANCEL_ABORTED
            }
            MessageTextId::RecruitmentCommandCancelUnknownSelection => {
                keys::RECRUITMENT_COMMAND_CANCEL_UNKNOWN_SELECTION
            }
            MessageTextId::RecruitmentCommandCancelTimeout => {
                keys::RECRUITMENT_COMMAND_CANCEL_TIMEOUT
            }
            MessageTextId::RecruitmentCommandCancelPermissionDenied => {
                keys::RECRUITMENT_COMMAND_CANCEL_PERMISSION_DENIED
            }
            MessageTextId::RecruitmentCommandChangePermissionDenied => {
                keys::RECRUITMENT_COMMAND_CHANGE_PERMISSION_DENIED
            }

            // Messages
            MessageTextId::MessagesWelcome => keys::MESSAGES_WELCOME,
            MessageTextId::MessagesHelp => keys::MESSAGES_HELP,
            MessageTextId::MessagesInitGuide => keys::MESSAGES_INIT_GUIDE,

            // Auto recruitment
            MessageTextId::AutoRecruitmentChannelCreateFailed => {
                keys::AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED
            }
            MessageTextId::AutoRecruitmentTimeSelectPlaceholder => {
                keys::AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER
            }
            MessageTextId::AutoRecruitmentUnregisterInCategoryError => {
                keys::AUTO_RECRUITMENT_UNREGISTER_IN_CATEGORY_ERROR
            }
            MessageTextId::AutoRecruitmentCategorySetupTimeSelectMessage => {
                keys::AUTO_RECRUITMENT_CATEGORY_SETUP_TIME_SELECT_MESSAGE
            }
            MessageTextId::AutoRecruitmentCategorySetupMatchingChannelMessage => {
                keys::AUTO_RECRUITMENT_CATEGORY_SETUP_MATCHING_CHANNEL_MESSAGE
            }
            MessageTextId::AutoRecruitmentCategorySetupQuestChannelEmptyMessage => {
                keys::AUTO_RECRUITMENT_CATEGORY_SETUP_QUEST_CHANNEL_EMPTY_MESSAGE
            }
            MessageTextId::AutoRecruitmentCategorySetupSelectionCheckButton => {
                keys::AUTO_RECRUITMENT_CATEGORY_SETUP_SELECTION_CHECK_BUTTON
            }
            MessageTextId::AutoRecruitmentCategorySetupSelectionCheckMessage => {
                keys::AUTO_RECRUITMENT_CATEGORY_SETUP_SELECTION_CHECK_MESSAGE
            }

            // App error
            MessageTextId::AppErrorDatabase => keys::APP_ERROR_DATABASE,
            MessageTextId::AppErrorDiscord => keys::APP_ERROR_DISCORD,
            MessageTextId::AppErrorConfig => keys::APP_ERROR_CONFIG,
            MessageTextId::AppErrorValidation => keys::APP_ERROR_VALIDATION,
            MessageTextId::AppErrorDiscordOperation => keys::APP_ERROR_DISCORD_OPERATION,
            MessageTextId::AppErrorChannelCreationFailed => keys::APP_ERROR_CHANNEL_CREATION_FAILED,
            MessageTextId::AppErrorInCategoryChannel => keys::APP_ERROR_IN_CATEGORY_CHANNEL,

            // Schedule command
            MessageTextId::ScheduleCommandGenerateLoading => {
                keys::SCHEDULE_COMMAND_GENERATE_LOADING
            }
            MessageTextId::ScheduleCommandGenerateSuccessTitle => {
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_TITLE
            }
            MessageTextId::ScheduleCommandGenerateSuccessDescription => {
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_DESCRIPTION
            }
            MessageTextId::ScheduleCommandSharedSuccessFieldName => {
                keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FIELD_NAME
            }
            MessageTextId::ScheduleCommandGenerateSuccessFieldValue => {
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_FIELD_VALUE
            }
            MessageTextId::ScheduleCommandSharedSuccessFooter => {
                keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FOOTER
            }
            MessageTextId::ScheduleCommandGenerateErrorTitle => {
                keys::SCHEDULE_COMMAND_GENERATE_ERROR_TITLE
            }
            MessageTextId::ScheduleCommandGenerateErrorDescription => {
                keys::SCHEDULE_COMMAND_GENERATE_ERROR_DESCRIPTION
            }
            MessageTextId::ScheduleCommandSharedErrorFooter => {
                keys::SCHEDULE_COMMAND_SHARED_ERROR_FOOTER
            }
            MessageTextId::ScheduleCommandGlobalGenerateLoading => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_LOADING
            }
            MessageTextId::ScheduleCommandGlobalGenerateSuccessTitle => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_TITLE
            }
            MessageTextId::ScheduleCommandGlobalGenerateSuccessDescription => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_DESCRIPTION
            }
            MessageTextId::ScheduleCommandGlobalGenerateSuccessFieldValue => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_FIELD_VALUE
            }
            MessageTextId::ScheduleCommandGlobalGenerateErrorTitle => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_TITLE
            }
            MessageTextId::ScheduleCommandGlobalGenerateErrorDescription => {
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_DESCRIPTION
            }
            MessageTextId::ScheduleCommandListTitle => keys::SCHEDULE_COMMAND_LIST_TITLE,
            MessageTextId::ScheduleCommandListEmptyDescription => {
                keys::SCHEDULE_COMMAND_LIST_EMPTY_DESCRIPTION
            }
            MessageTextId::ScheduleCommandListFooter => keys::SCHEDULE_COMMAND_LIST_FOOTER,
            MessageTextId::ScheduleCommandHistoryTitle => keys::SCHEDULE_COMMAND_HISTORY_TITLE,
            MessageTextId::ScheduleCommandHistoryEmptyDescription => {
                keys::SCHEDULE_COMMAND_HISTORY_EMPTY_DESCRIPTION
            }
            MessageTextId::ScheduleCommandHistoryTitleWithDays => {
                keys::SCHEDULE_COMMAND_HISTORY_TITLE_WITH_DAYS
            }
            MessageTextId::ScheduleCommandHistoryFooter => keys::SCHEDULE_COMMAND_HISTORY_FOOTER,
            MessageTextId::ScheduleCommandStatsTitleWithDays => {
                keys::SCHEDULE_COMMAND_STATS_TITLE_WITH_DAYS
            }
            MessageTextId::ScheduleCommandStatsFooter => keys::SCHEDULE_COMMAND_STATS_FOOTER,
            MessageTextId::ScheduleCommandStatsDescriptionHeader => {
                keys::SCHEDULE_COMMAND_STATS_DESCRIPTION_HEADER
            }
            MessageTextId::ScheduleCommandStatsMessageTypeHeader => {
                keys::SCHEDULE_COMMAND_STATS_MESSAGE_TYPE_HEADER
            }
            MessageTextId::ScheduleCommandStatsOtherTypes => {
                keys::SCHEDULE_COMMAND_STATS_OTHER_TYPES
            }

            // Recruitment schedule command
            MessageTextId::RecruitmentScheduleListEmptyAll => {
                keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_ALL
            }
            MessageTextId::RecruitmentScheduleListEmptySelf => {
                keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_SELF
            }
            MessageTextId::RecruitmentScheduleListTitleAll => {
                keys::RECRUITMENT_SCHEDULE_LIST_TITLE_ALL
            }
            MessageTextId::RecruitmentScheduleListTitleSelf => {
                keys::RECRUITMENT_SCHEDULE_LIST_TITLE_SELF
            }
            MessageTextId::RecruitmentScheduleListStatusEnabled => {
                keys::RECRUITMENT_SCHEDULE_LIST_STATUS_ENABLED
            }
            MessageTextId::RecruitmentScheduleListStatusDisabled => {
                keys::RECRUITMENT_SCHEDULE_LIST_STATUS_DISABLED
            }
            MessageTextId::RecruitmentScheduleListDismissalPrefix => {
                keys::RECRUITMENT_SCHEDULE_LIST_DISMISSAL_PREFIX
            }
            MessageTextId::RecruitmentScheduleListMoreCount => {
                keys::RECRUITMENT_SCHEDULE_LIST_MORE_COUNT
            }
            MessageTextId::RecruitmentScheduleListFooter => keys::RECRUITMENT_SCHEDULE_LIST_FOOTER,
            MessageTextId::RecruitmentListTitle => keys::RECRUITMENT_LIST_TITLE,
            MessageTextId::RecruitmentListEmpty => keys::RECRUITMENT_LIST_EMPTY,
            MessageTextId::RecruitmentListLinkText => keys::RECRUITMENT_LIST_LINK_TEXT,
            MessageTextId::RecruitmentListMoreCount => keys::RECRUITMENT_LIST_MORE_COUNT,
            MessageTextId::RecruitmentListFooter => keys::RECRUITMENT_LIST_FOOTER,
            MessageTextId::QuestListTitleAll => keys::QUEST_LIST_TITLE_ALL,
            MessageTextId::QuestListTitleEnabled => keys::QUEST_LIST_TITLE_ENABLED,
            MessageTextId::QuestListTitleDisabled => keys::QUEST_LIST_TITLE_DISABLED,
            MessageTextId::QuestListMoreCount => keys::QUEST_LIST_MORE_COUNT,
            MessageTextId::QuestListEmptyEnabled => keys::QUEST_LIST_EMPTY_ENABLED,
            MessageTextId::QuestListEmptyDisabled => keys::QUEST_LIST_EMPTY_DISABLED,
            MessageTextId::RecruitmentScheduleToggleSuccessTitle => {
                keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_TITLE
            }
            MessageTextId::RecruitmentScheduleToggleSuccessDescription => {
                keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_DESCRIPTION
            }
            MessageTextId::RecruitmentScheduleDeleteSuccessTitle => {
                keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_TITLE
            }
            MessageTextId::RecruitmentScheduleDeleteSuccessDescription => {
                keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_DESCRIPTION
            }

            // Recruitment role show
            MessageTextId::RecruitmentRoleShowNotRegistered => {
                keys::RECRUITMENT_ROLE_SHOW_NOT_REGISTERED
            }
            MessageTextId::RecruitmentRoleShowHeader => keys::RECRUITMENT_ROLE_SHOW_HEADER,
            MessageTextId::RecruitmentRoleShowSectionAll => keys::RECRUITMENT_ROLE_SHOW_SECTION_ALL,
            MessageTextId::RecruitmentRoleShowSectionQuest => {
                keys::RECRUITMENT_ROLE_SHOW_SECTION_QUEST
            }
            MessageTextId::RecruitmentRoleShowUnknownQuest => {
                keys::RECRUITMENT_ROLE_SHOW_UNKNOWN_QUEST
            }

            // Recruitment change panel
            MessageTextId::RecruitmentCommandChangePanelUnchanged => {
                keys::RECRUITMENT_COMMAND_CHANGE_PANEL_UNCHANGED
            }
            MessageTextId::RecruitmentCommandChangePanelContent => {
                keys::RECRUITMENT_COMMAND_CHANGE_PANEL_CONTENT
            }
            MessageTextId::RecruitmentCommandChangeOptionQuestUnchanged => {
                keys::RECRUITMENT_COMMAND_CHANGE_OPTION_QUEST_UNCHANGED
            }
            MessageTextId::RecruitmentCommandChangeOptionStyleUnchanged => {
                keys::RECRUITMENT_COMMAND_CHANGE_OPTION_STYLE_UNCHANGED
            }
            MessageTextId::RecruitmentCommandChangePlaceholderQuest => {
                keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_QUEST
            }
            MessageTextId::RecruitmentCommandChangePlaceholderStyle => {
                keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_STYLE
            }
            MessageTextId::RecruitmentCommandChangeButtonOpenDate => {
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_OPEN_DATE
            }
            MessageTextId::RecruitmentCommandChangeButtonClearDate => {
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_CLEAR_DATE
            }
            MessageTextId::RecruitmentCommandChangeButtonApply => {
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_APPLY
            }
            MessageTextId::RecruitmentCommandChangeModalTitle => {
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_TITLE
            }
            MessageTextId::RecruitmentCommandChangeModalEventDateLabel => {
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_LABEL
            }
            MessageTextId::RecruitmentCommandChangeModalEventDatePlaceholder => {
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_PLACEHOLDER
            }
            MessageTextId::RecruitmentCommandChangeModalAbsoluteDatetimeRequired => {
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_ABSOLUTE_DATETIME_REQUIRED
            }
            MessageTextId::RecruitmentCommandChangeModalParseFailed => {
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_PARSE_FAILED
            }

            // Channel show
            MessageTextId::ChannelShowEmpty => keys::CHANNEL_SHOW_EMPTY,
            MessageTextId::ChannelShowHeader => keys::CHANNEL_SHOW_HEADER,
            MessageTextId::ChannelShowUnset => keys::CHANNEL_SHOW_UNSET,

            // Auto recruitment interaction
            MessageTextId::AutoRecruitmentQuestSelectRequired => {
                keys::AUTO_RECRUITMENT_QUEST_SELECT_REQUIRED
            }
            MessageTextId::AutoRecruitmentQuestSelectRegistered => {
                keys::AUTO_RECRUITMENT_QUEST_SELECT_REGISTERED
            }
            MessageTextId::AutoRecruitmentTimeSelectRequired => {
                keys::AUTO_RECRUITMENT_TIME_SELECT_REQUIRED
            }
            MessageTextId::AutoRecruitmentTimeSelectRegistered => {
                keys::AUTO_RECRUITMENT_TIME_SELECT_REGISTERED
            }
            MessageTextId::AutoRecruitmentStatusHeader => keys::AUTO_RECRUITMENT_STATUS_HEADER,
            MessageTextId::AutoRecruitmentStatusQuestEmpty => {
                keys::AUTO_RECRUITMENT_STATUS_QUEST_EMPTY
            }
            MessageTextId::AutoRecruitmentStatusQuestCount => {
                keys::AUTO_RECRUITMENT_STATUS_QUEST_COUNT
            }
            MessageTextId::AutoRecruitmentStatusQuestIds => keys::AUTO_RECRUITMENT_STATUS_QUEST_IDS,
            MessageTextId::AutoRecruitmentStatusTimeEmpty => {
                keys::AUTO_RECRUITMENT_STATUS_TIME_EMPTY
            }
            MessageTextId::AutoRecruitmentStatusTimeHeader => {
                keys::AUTO_RECRUITMENT_STATUS_TIME_HEADER
            }
            MessageTextId::AutoRecruitmentStatusTimeSlot => keys::AUTO_RECRUITMENT_STATUS_TIME_SLOT,
            MessageTextId::AutoRecruitmentPresenterElementFire => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_FIRE
            }
            MessageTextId::AutoRecruitmentPresenterElementWater => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_WATER
            }
            MessageTextId::AutoRecruitmentPresenterElementEarth => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_EARTH
            }
            MessageTextId::AutoRecruitmentPresenterElementWind => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_WIND
            }
            MessageTextId::AutoRecruitmentPresenterElementLight => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_LIGHT
            }
            MessageTextId::AutoRecruitmentPresenterElementDark => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_DARK
            }
            MessageTextId::AutoRecruitmentPresenterJoinButton => {
                keys::AUTO_RECRUITMENT_PRESENTER_JOIN_BUTTON
            }
            MessageTextId::AutoRecruitmentPresenterElementPlaceholder => {
                keys::AUTO_RECRUITMENT_PRESENTER_ELEMENT_PLACEHOLDER
            }
            MessageTextId::AutoRecruitmentPresenterQuestSelectPlaceholder => {
                keys::AUTO_RECRUITMENT_PRESENTER_QUEST_SELECT_PLACEHOLDER
            }
            MessageTextId::AutoRecruitmentPresenterQuestSelectMessage => {
                keys::AUTO_RECRUITMENT_PRESENTER_QUEST_SELECT_MESSAGE
            }
            MessageTextId::AutoRecruitmentPresenterTimeSelectPlaceholder => {
                keys::AUTO_RECRUITMENT_PRESENTER_TIME_SELECT_PLACEHOLDER
            }
            MessageTextId::AutoRecruitmentPresenterSetupCompleteTitle => {
                keys::AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_TITLE
            }
            MessageTextId::AutoRecruitmentPresenterSetupCompleteDescription => {
                keys::AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_DESCRIPTION
            }
            MessageTextId::AutoRecruitmentPresenterSetupCompleteQuestField => {
                keys::AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_QUEST_FIELD
            }
            MessageTextId::AutoRecruitmentPresenterSetupCompleteTimeField => {
                keys::AUTO_RECRUITMENT_PRESENTER_SETUP_COMPLETE_TIME_FIELD
            }
            // Help embed
            MessageTextId::HelpEmbedTitle => keys::HELP_EMBED_TITLE,
            MessageTextId::HelpEmbedDescription => keys::HELP_EMBED_DESCRIPTION,
            MessageTextId::HelpEmbedCommandsFieldTitle => keys::HELP_EMBED_COMMANDS_FIELD_TITLE,
            MessageTextId::HelpEmbedCommandsFieldValue => keys::HELP_EMBED_COMMANDS_FIELD_VALUE,
            MessageTextId::HelpEmbedRecruitFieldValue => keys::HELP_EMBED_RECRUIT_FIELD_VALUE,
            MessageTextId::HelpEmbedEnvironLoadFieldValue => {
                keys::HELP_EMBED_ENVIRON_LOAD_FIELD_VALUE
            }
            MessageTextId::HelpEmbedGspreadLoadFieldValue => {
                keys::HELP_EMBED_GSPREAD_LOAD_FIELD_VALUE
            }
            MessageTextId::HelpEmbedGspreadPushFieldValue => {
                keys::HELP_EMBED_GSPREAD_PUSH_FIELD_VALUE
            }
            MessageTextId::HelpEmbedHelpFieldValue => keys::HELP_EMBED_HELP_FIELD_VALUE,
            MessageTextId::HelpEmbedFooter => keys::HELP_EMBED_FOOTER,
        }
    }
}

impl std::fmt::Display for MessageTextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::fs;

    #[test]
    fn test_message_id_as_str() {
        assert_eq!(
            MessageTextId::TimezoneShowCurrent.as_str(),
            "timezone.show_current"
        );
        assert_eq!(
            MessageTextId::TimezoneSetSuccess.as_str(),
            "timezone.set_success"
        );
        assert_eq!(MessageTextId::ErrorsGuildOnly.as_str(), "errors.guild_only");
        assert_eq!(
            MessageTextId::ScheduleCommandSharedSuccessFieldName.as_str(),
            "schedule.command.shared.success_field_name"
        );
        assert_eq!(
            MessageTextId::ScheduleCommandSharedSuccessFooter.as_str(),
            "schedule.command.shared.success_footer"
        );
        assert_eq!(
            MessageTextId::ScheduleCommandSharedErrorFooter.as_str(),
            "schedule.command.shared.error_footer"
        );
        assert_eq!(
            MessageTextId::RecruitmentCommandChangePanelUnchanged.as_str(),
            "recruitment.command.change.panel_unchanged"
        );
        assert_eq!(
            MessageTextId::CommonErrorPrefix.as_str(),
            "common.error_prefix"
        );
    }

    #[test]
    fn test_message_id_display() {
        assert_eq!(
            MessageTextId::TimezoneShowCurrent.to_string(),
            "timezone.show_current"
        );
    }

    #[test]
    fn test_renamed_old_ids_are_not_resolved_in_yaml_loader() {
        let old_ids = [
            "schedule.command.generate.success_field_name",
            "schedule.command.global_generate.success_field_name",
            "schedule.command.generate.success_footer",
            "schedule.command.global_generate.success_footer",
            "schedule.command.generate.error_footer",
            "schedule.command.global_generate.error_footer",
            "recruitment.command.change.panel_quest_unchanged",
            "recruitment.command.change.panel_style_unchanged",
            "recruitment.command.change.panel_date_unchanged",
            "recruitment.command.change.error_prefix",
            "auto_recruitment.operation_error_prefix",
        ];

        for old_id in old_ids {
            let resolved = super::super::yaml_loader::get_yaml_message(old_id, "ja");
            assert!(
                resolved.is_none(),
                "旧IDが解決されました: old_id={old_id}, resolved={resolved:?}"
            );
        }
    }

    #[test]
    fn test_new_shared_ids_are_resolved_in_yaml_loader() {
        let new_ids = [
            "schedule.command.shared.success_field_name",
            "schedule.command.shared.success_footer",
            "schedule.command.shared.error_footer",
            "recruitment.command.change.panel_unchanged",
            "common.error_prefix",
        ];

        for new_id in new_ids {
            let resolved = super::super::yaml_loader::get_yaml_message(new_id, "ja");
            assert!(resolved.is_some(), "新IDが解決できません: new_id={new_id}");
        }
    }

    fn parse_yaml_message_entries(yaml_content: &str) -> Vec<(String, String, String)> {
        let mut entries = Vec::new();
        let mut current_key: Option<String> = None;
        let mut current_ja: Option<String> = None;
        let mut current_en: Option<String> = None;

        let flush_entry = |entries: &mut Vec<(String, String, String)>,
                           current_key: &mut Option<String>,
                           current_ja: &mut Option<String>,
                           current_en: &mut Option<String>| {
            if let (Some(key), Some(ja), Some(en)) =
                (current_key.take(), current_ja.take(), current_en.take())
            {
                entries.push((key, ja, en));
            } else {
                current_key.take();
                current_ja.take();
                current_en.take();
            }
        };

        let parse_yaml_value = |line: &str, field_prefix: &str| -> Option<String> {
            let trimmed = line.trim_start();
            let value = trimmed.strip_prefix(field_prefix)?.trim();
            if value.is_empty() {
                return Some(String::new());
            }
            if value.len() >= 2
                && ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
            {
                return Some(value[1..value.len() - 1].to_string());
            }
            Some(value.to_string())
        };

        for line in yaml_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let is_top_level_key =
                !line.starts_with(' ') && trimmed.ends_with(':') && trimmed != "_version:";

            if is_top_level_key {
                flush_entry(
                    &mut entries,
                    &mut current_key,
                    &mut current_ja,
                    &mut current_en,
                );
                current_key = Some(trimmed.trim_end_matches(':').to_string());
                continue;
            }

            if let Some(ja) = parse_yaml_value(line, "ja:") {
                current_ja = Some(ja);
                continue;
            }
            if let Some(en) = parse_yaml_value(line, "en:") {
                current_en = Some(en);
            }
        }

        flush_entry(
            &mut entries,
            &mut current_key,
            &mut current_ja,
            &mut current_en,
        );
        entries
    }

    fn normalize_for_alphabet_level_match(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }

    #[test]
    fn test_no_identical_ja_en_entries_in_messages_yaml() {
        let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/locales/messages.yml");
        let yaml_content =
            fs::read_to_string(yaml_path).expect("locales/messages.yml が見つかりません");

        let entries = parse_yaml_message_entries(&yaml_content);
        let same_entries: Vec<String> = entries
            .iter()
            .filter_map(|(key, ja, en)| if ja == en { Some(key.clone()) } else { None })
            .collect();

        assert!(
            same_entries.is_empty(),
            "ja/en が同一のキーがあります: {}",
            same_entries.join(", ")
        );
    }

    #[test]
    fn test_no_duplicate_ja_en_pairs_in_messages_yaml() {
        let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/locales/messages.yml");
        let yaml_content =
            fs::read_to_string(yaml_path).expect("locales/messages.yml が見つかりません");

        let entries = parse_yaml_message_entries(&yaml_content);
        let mut seen_pairs: HashMap<(String, String), String> = HashMap::new();
        let mut duplicates = Vec::new();

        for (key, ja, en) in entries {
            let pair = (ja.clone(), en.clone());
            if let Some(existing_key) = seen_pairs.get(&pair) {
                duplicates.push(format!("{existing_key} <-> {key}"));
            } else {
                seen_pairs.insert(pair, key);
            }
        }

        assert!(
            duplicates.is_empty(),
            "ja/en 完全一致ペアの重複キーがあります: {}",
            duplicates.join(", ")
        );
    }

    /// MessageTextIdの全enumバリアントがYAMLファイルに定義されていることを検証する
    ///
    /// このテストは以下を保証する:
    /// - Rustコードで定義された全てのメッセージIDが、対応するYAMLキーを持つこと
    /// - YAMLファイルに存在しないキーを参照しようとした場合、テストが失敗すること
    ///
    /// YAMLに余分なキーがあっても問題ない（将来の拡張のため）
    #[test]
    fn test_all_message_ids_exist_in_yaml() {
        // YAMLファイルを読み込む
        let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/locales/messages.yml");
        let yaml_content =
            fs::read_to_string(yaml_path).expect("locales/messages.yml が見つかりません");

        // YAMLから全てのキーを抽出
        let mut yaml_keys = HashSet::new();
        for line in yaml_content.lines() {
            // "key:" の形式の行を抽出（インデントは無視）
            if let Some(key) = line.trim_start().strip_suffix(':') {
                // "_version" や言語キー（"ja:", "en:"）は除外
                if !key.starts_with('_') && key != "ja" && key != "en" {
                    yaml_keys.insert(key.to_string());
                }
            }
        }

        // 全てのMessageTextIdバリアントを列挙
        let all_message_ids = vec![
            MessageTextId::CommonSuccess,
            MessageTextId::CommonError,
            MessageTextId::CommonErrorPrefix,
            MessageTextId::CommonWarning,
            MessageTextId::CommonInfo,
            MessageTextId::CommonYes,
            MessageTextId::CommonNo,
            MessageTextId::CommonCancel,
            MessageTextId::CommonConfirm,
            MessageTextId::CommonLoading,
            MessageTextId::CommonUnknown,
            MessageTextId::RecruitmentUiTitle,
            MessageTextId::RecruitmentUiNewRecruitment,
            MessageTextId::RecruitmentUiRecruitmentCancelled,
            MessageTextId::RecruitmentUiRecruitmentClosed,
            MessageTextId::RecruitmentUiRecruitmentFull,
            MessageTextId::RecruitmentUiJoinSuccess,
            MessageTextId::RecruitmentUiLeaveSuccess,
            MessageTextId::RecruitmentUiNotFound,
            MessageTextId::RecruitmentUiAlreadyJoined,
            MessageTextId::RecruitmentUiNotJoined,
            MessageTextId::ErrorsInvalidInput,
            MessageTextId::ErrorsPermissionDenied,
            MessageTextId::ErrorsInternalError,
            MessageTextId::ErrorsUserNotFound,
            MessageTextId::ErrorsCommandFailed,
            MessageTextId::ErrorsEnvVarNotSet,
            MessageTextId::ErrorsGuildOnly,
            MessageTextId::ErrorsSpreadsheetNotRegistered,
            MessageTextId::ErrorsSpreadsheetConfigFetchFailed,
            MessageTextId::SpreadsheetLoading,
            MessageTextId::SpreadsheetLoadSuccess,
            MessageTextId::SpreadsheetLoadPartialSuccess,
            MessageTextId::SpreadsheetLoadFailed,
            MessageTextId::SpreadsheetRegistering,
            MessageTextId::SpreadsheetRegisterSuccess,
            MessageTextId::SpreadsheetRegisterFailed,
            MessageTextId::SpreadsheetPushing,
            MessageTextId::SpreadsheetPushSuccess,
            MessageTextId::SpreadsheetPushPartialSuccess,
            MessageTextId::SpreadsheetPushFailed,
            MessageTextId::SpreadsheetGlobalPushing,
            MessageTextId::SpreadsheetGlobalPushSuccess,
            MessageTextId::SpreadsheetGlobalPushPartialSuccess,
            MessageTextId::SpreadsheetGlobalPushFailed,
            MessageTextId::SpreadsheetGlobalLoading,
            MessageTextId::SpreadsheetGlobalLoadSuccess,
            MessageTextId::SpreadsheetGlobalLoadFailed,
            MessageTextId::KosenjoBefore3Days,
            MessageTextId::KosenjoBefore1Day,
            MessageTextId::KosenjoQualifyingStart,
            MessageTextId::KosenjoQualifyingEnd,
            MessageTextId::KosenjoQualifyingEndNoInterval,
            MessageTextId::KosenjoMainTournamentBefore1Day,
            MessageTextId::KosenjoMainTournamentDayStart,
            MessageTextId::KosenjoMainTournamentHalfDay,
            MessageTextId::KosenjoMainTournamentDayEnd,
            MessageTextId::KosenjoMainTournamentEnd,
            MessageTextId::KosenjoSpBattleEnd,
            MessageTextId::KosenjoTeamAbility1,
            MessageTextId::KosenjoTeamAbility2,
            MessageTextId::DorebaraStart,
            MessageTextId::DorebaraEnd,
            MessageTextId::DorebaraReset,
            MessageTextId::DorebaraVariant,
            MessageTextId::DorebaraLastDay,
            MessageTextId::BotMention,
            MessageTextId::BotMentionSix,
            MessageTextId::BotMentionCalling,
            MessageTextId::OmikujiHihi,
            MessageTextId::OmikujiHakyoku,
            MessageTextId::OmikujiOmegaUnit,
            MessageTextId::RecruitmentDisplayNormal,
            MessageTextId::RecruitmentDisplaySixElements,
            MessageTextId::RecruitmentDisplayEventDateLabel,
            MessageTextId::RecruitmentDisplayDateFormat,
            MessageTextId::RecruitmentDisplayDismissalTimesLabel,
            MessageTextId::RecruitmentDisplayElementFire,
            MessageTextId::RecruitmentDisplayElementWater,
            MessageTextId::RecruitmentDisplayElementEarth,
            MessageTextId::RecruitmentDisplayElementWind,
            MessageTextId::RecruitmentDisplayElementLight,
            MessageTextId::RecruitmentDisplayElementDark,
            MessageTextId::RecruitmentDisplayAllElements,
            MessageTextId::RecruitmentDisplayNoParticipants,
            MessageTextId::RecruitmentDisplayLeaveAllButton,
            MessageTextId::RecruitmentNotificationMemberFull,
            MessageTextId::RecruitmentNotificationBefore5Minutes,
            MessageTextId::RecruitmentNotificationStart,
            MessageTextId::RecruitmentNotificationDismissal,
            MessageTextId::RecruitmentNotificationDismissalWithParticipants,
            MessageTextId::TimezoneSetSuccess,
            MessageTextId::TimezoneShowCurrent,
            MessageTextId::GuildSettingsSetSuccess,
            MessageTextId::GuildSettingsShowSuccess,
            MessageTextId::GuildSettingsNotSet,
            MessageTextId::RecruitmentRoleAddSuccess,
            MessageTextId::RecruitmentRoleRemoveSuccess,
            MessageTextId::RecruitmentCommandCancelAlreadyCancelled,
            MessageTextId::RecruitmentCommandCancelMessageDeleted,
            MessageTextId::RecruitmentCommandCancelInvalidMessage,
            MessageTextId::RecruitmentCommandCancelNotFound,
            MessageTextId::RecruitmentCommandCancelEventDatePassed,
            MessageTextId::RecruitmentCommandCancelError,
            MessageTextId::RecruitmentCommandCancelledMessageSuffix,
            MessageTextId::RecruitmentCommandCancelNotificationNoParticipants,
            MessageTextId::RecruitmentCommandCancelNotificationWithParticipants,
            MessageTextId::RecruitmentCommandCancellingProgress,
            MessageTextId::RecruitmentCommandChangeNoChanges,
            MessageTextId::RecruitmentCommandChangeSuccess,
            MessageTextId::RecruitmentCommandCancelConfirmPrompt,
            MessageTextId::RecruitmentCommandCancelAborted,
            MessageTextId::RecruitmentCommandCancelUnknownSelection,
            MessageTextId::RecruitmentCommandCancelTimeout,
            MessageTextId::RecruitmentCommandCancelPermissionDenied,
            MessageTextId::RecruitmentCommandChangePermissionDenied,
            MessageTextId::MessagesWelcome,
            MessageTextId::MessagesHelp,
            MessageTextId::MessagesInitGuide,
            MessageTextId::AutoRecruitmentChannelCreateFailed,
            MessageTextId::AutoRecruitmentTimeSelectPlaceholder,
            MessageTextId::AutoRecruitmentUnregisterInCategoryError,
            MessageTextId::AutoRecruitmentCategorySetupTimeSelectMessage,
            MessageTextId::AutoRecruitmentCategorySetupMatchingChannelMessage,
            MessageTextId::AutoRecruitmentCategorySetupQuestChannelEmptyMessage,
            MessageTextId::AutoRecruitmentCategorySetupSelectionCheckButton,
            MessageTextId::AutoRecruitmentCategorySetupSelectionCheckMessage,
            MessageTextId::AutoRecruitmentStatusHeader,
            MessageTextId::AutoRecruitmentStatusQuestEmpty,
            MessageTextId::AutoRecruitmentStatusQuestCount,
            MessageTextId::AutoRecruitmentStatusQuestIds,
            MessageTextId::AutoRecruitmentStatusTimeEmpty,
            MessageTextId::AutoRecruitmentStatusTimeHeader,
            MessageTextId::AutoRecruitmentStatusTimeSlot,
            MessageTextId::AutoRecruitmentPresenterElementFire,
            MessageTextId::AutoRecruitmentPresenterElementWater,
            MessageTextId::AutoRecruitmentPresenterElementEarth,
            MessageTextId::AutoRecruitmentPresenterElementWind,
            MessageTextId::AutoRecruitmentPresenterElementLight,
            MessageTextId::AutoRecruitmentPresenterElementDark,
            MessageTextId::AutoRecruitmentPresenterJoinButton,
            MessageTextId::AutoRecruitmentPresenterElementPlaceholder,
            MessageTextId::AutoRecruitmentPresenterQuestSelectPlaceholder,
            MessageTextId::AutoRecruitmentPresenterQuestSelectMessage,
            MessageTextId::AutoRecruitmentPresenterTimeSelectPlaceholder,
            MessageTextId::AutoRecruitmentPresenterSetupCompleteTitle,
            MessageTextId::AutoRecruitmentPresenterSetupCompleteDescription,
            MessageTextId::AutoRecruitmentPresenterSetupCompleteQuestField,
            MessageTextId::AutoRecruitmentPresenterSetupCompleteTimeField,
            MessageTextId::AppErrorDatabase,
            MessageTextId::AppErrorDiscord,
            MessageTextId::AppErrorConfig,
            MessageTextId::AppErrorValidation,
            MessageTextId::AppErrorDiscordOperation,
            MessageTextId::AppErrorChannelCreationFailed,
            MessageTextId::AppErrorInCategoryChannel,
            MessageTextId::ScheduleCommandGenerateLoading,
            MessageTextId::ScheduleCommandGenerateSuccessTitle,
            MessageTextId::ScheduleCommandGenerateSuccessDescription,
            MessageTextId::ScheduleCommandSharedSuccessFieldName,
            MessageTextId::ScheduleCommandGenerateSuccessFieldValue,
            MessageTextId::ScheduleCommandSharedSuccessFooter,
            MessageTextId::ScheduleCommandGenerateErrorTitle,
            MessageTextId::ScheduleCommandGenerateErrorDescription,
            MessageTextId::ScheduleCommandSharedErrorFooter,
            MessageTextId::ScheduleCommandGlobalGenerateLoading,
            MessageTextId::ScheduleCommandGlobalGenerateSuccessTitle,
            MessageTextId::ScheduleCommandGlobalGenerateSuccessDescription,
            MessageTextId::ScheduleCommandGlobalGenerateSuccessFieldValue,
            MessageTextId::ScheduleCommandGlobalGenerateErrorTitle,
            MessageTextId::ScheduleCommandGlobalGenerateErrorDescription,
            MessageTextId::ScheduleCommandListTitle,
            MessageTextId::ScheduleCommandListEmptyDescription,
            MessageTextId::ScheduleCommandListFooter,
            MessageTextId::ScheduleCommandHistoryTitle,
            MessageTextId::ScheduleCommandHistoryEmptyDescription,
            MessageTextId::ScheduleCommandHistoryTitleWithDays,
            MessageTextId::ScheduleCommandHistoryFooter,
            MessageTextId::ScheduleCommandStatsTitleWithDays,
            MessageTextId::ScheduleCommandStatsFooter,
            MessageTextId::ScheduleCommandStatsDescriptionHeader,
            MessageTextId::ScheduleCommandStatsMessageTypeHeader,
            MessageTextId::ScheduleCommandStatsOtherTypes,
            MessageTextId::RecruitmentScheduleListEmptyAll,
            MessageTextId::RecruitmentScheduleListEmptySelf,
            MessageTextId::RecruitmentScheduleListTitleAll,
            MessageTextId::RecruitmentScheduleListTitleSelf,
            MessageTextId::RecruitmentScheduleListStatusEnabled,
            MessageTextId::RecruitmentScheduleListStatusDisabled,
            MessageTextId::RecruitmentScheduleListDismissalPrefix,
            MessageTextId::RecruitmentScheduleListMoreCount,
            MessageTextId::RecruitmentScheduleListFooter,
            MessageTextId::RecruitmentListTitle,
            MessageTextId::RecruitmentListEmpty,
            MessageTextId::RecruitmentListLinkText,
            MessageTextId::RecruitmentListMoreCount,
            MessageTextId::RecruitmentListFooter,
            MessageTextId::QuestListTitleAll,
            MessageTextId::QuestListTitleEnabled,
            MessageTextId::QuestListTitleDisabled,
            MessageTextId::QuestListMoreCount,
            MessageTextId::QuestListEmptyEnabled,
            MessageTextId::QuestListEmptyDisabled,
            MessageTextId::RecruitmentScheduleToggleSuccessTitle,
            MessageTextId::RecruitmentScheduleToggleSuccessDescription,
            MessageTextId::RecruitmentScheduleDeleteSuccessTitle,
            MessageTextId::RecruitmentScheduleDeleteSuccessDescription,
            MessageTextId::RecruitmentRoleShowNotRegistered,
            MessageTextId::RecruitmentRoleShowHeader,
            MessageTextId::RecruitmentRoleShowSectionAll,
            MessageTextId::RecruitmentRoleShowSectionQuest,
            MessageTextId::RecruitmentRoleShowUnknownQuest,
            MessageTextId::RecruitmentCommandChangePanelUnchanged,
            MessageTextId::RecruitmentCommandChangePanelContent,
            MessageTextId::RecruitmentCommandChangeOptionQuestUnchanged,
            MessageTextId::RecruitmentCommandChangeOptionStyleUnchanged,
            MessageTextId::RecruitmentCommandChangePlaceholderQuest,
            MessageTextId::RecruitmentCommandChangePlaceholderStyle,
            MessageTextId::RecruitmentCommandChangeButtonOpenDate,
            MessageTextId::RecruitmentCommandChangeButtonClearDate,
            MessageTextId::RecruitmentCommandChangeButtonApply,
            MessageTextId::RecruitmentCommandChangeModalTitle,
            MessageTextId::RecruitmentCommandChangeModalEventDateLabel,
            MessageTextId::RecruitmentCommandChangeModalEventDatePlaceholder,
            MessageTextId::RecruitmentCommandChangeModalAbsoluteDatetimeRequired,
            MessageTextId::RecruitmentCommandChangeModalParseFailed,
            MessageTextId::ChannelShowEmpty,
            MessageTextId::ChannelShowHeader,
            MessageTextId::ChannelShowUnset,
            MessageTextId::AutoRecruitmentQuestSelectRequired,
            MessageTextId::AutoRecruitmentQuestSelectRegistered,
            MessageTextId::AutoRecruitmentTimeSelectRequired,
            MessageTextId::AutoRecruitmentTimeSelectRegistered,
            MessageTextId::HelpEmbedTitle,
            MessageTextId::HelpEmbedDescription,
            MessageTextId::HelpEmbedCommandsFieldTitle,
            MessageTextId::HelpEmbedCommandsFieldValue,
            MessageTextId::HelpEmbedRecruitFieldValue,
            MessageTextId::HelpEmbedEnvironLoadFieldValue,
            MessageTextId::HelpEmbedGspreadLoadFieldValue,
            MessageTextId::HelpEmbedGspreadPushFieldValue,
            MessageTextId::HelpEmbedHelpFieldValue,
            MessageTextId::HelpEmbedFooter,
        ];

        let mut unique_message_ids = HashSet::new();
        for message_id in &all_message_ids {
            assert!(
                unique_message_ids.insert(*message_id),
                "all_message_ids に重複があります: {message_id:?}"
            );
        }

        for message_id in &all_message_ids {
            let variant_name = format!("{message_id:?}");
            let normalized_variant = normalize_for_alphabet_level_match(&variant_name);
            let normalized_key = normalize_for_alphabet_level_match(message_id.as_str());
            assert_eq!(
                normalized_variant,
                normalized_key,
                "MessageTextId と YAML キーのアルファベット一致に失敗: variant={variant_name}, key={}",
                message_id.as_str()
            );
        }

        // 各MessageTextIdに対してYAMLにキーが存在することを確認
        let mut missing_keys = Vec::new();
        for message_id in &all_message_ids {
            let key = message_id.as_str();
            if !yaml_keys.contains(key) {
                missing_keys.push(key.to_string());
            }
        }

        // 見つからないキーがあった場合、テスト失敗
        if !missing_keys.is_empty() {
            panic!(
                "以下のメッセージIDがYAMLファイルに定義されていません:\n{}",
                missing_keys.join("\n")
            );
        }
    }
}
