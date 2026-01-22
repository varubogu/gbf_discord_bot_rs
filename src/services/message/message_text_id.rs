/// メッセージキー定数定義
///
/// YAMLファイル内のキーを定数として定義する。
/// これらの定数は `MessageTextId::as_str()` と `yaml_loader.rs` の両方から参照される。
/// 単一の情報源(Single Source of Truth)として機能する。
pub mod keys {
    // Common messages
    pub const COMMON_SUCCESS: &str = "common.success";
    pub const COMMON_ERROR: &str = "common.error";
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

    // General messages
    pub const MESSAGES_WELCOME: &str = "messages.welcome";
    pub const MESSAGES_HELP: &str = "messages.help";

    // Auto recruitment messages
    pub const AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED: &str =
        "auto_recruitment.channel_create_failed";
    pub const AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER: &str =
        "auto_recruitment.time_select_placeholder";
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

    // General messages
    MessagesWelcome,
    MessagesHelp,

    // Auto recruitment messages
    AutoRecruitmentChannelCreateFailed,
    AutoRecruitmentTimeSelectPlaceholder,
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

            // Messages
            MessageTextId::MessagesWelcome => keys::MESSAGES_WELCOME,
            MessageTextId::MessagesHelp => keys::MESSAGES_HELP,

            // Auto recruitment
            MessageTextId::AutoRecruitmentChannelCreateFailed => {
                keys::AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED
            }
            MessageTextId::AutoRecruitmentTimeSelectPlaceholder => {
                keys::AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER
            }
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
    use std::collections::HashSet;
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
    }

    #[test]
    fn test_message_id_display() {
        assert_eq!(
            MessageTextId::TimezoneShowCurrent.to_string(),
            "timezone.show_current"
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
            MessageTextId::MessagesWelcome,
            MessageTextId::MessagesHelp,
            MessageTextId::AutoRecruitmentChannelCreateFailed,
            MessageTextId::AutoRecruitmentTimeSelectPlaceholder,
        ];

        // 各MessageTextIdに対してYAMLにキーが存在することを確認
        let mut missing_keys = Vec::new();
        for message_id in all_message_ids {
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
