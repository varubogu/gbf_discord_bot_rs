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

    // Battle recruitment messages
    BattleRecruitmentTitle,
    BattleRecruitmentNewRecruitment,
    BattleRecruitmentRecruitmentCancelled,
    BattleRecruitmentRecruitmentClosed,
    BattleRecruitmentRecruitmentFull,
    BattleRecruitmentJoinSuccess,
    BattleRecruitmentLeaveSuccess,
    BattleRecruitmentNotFound,
    BattleRecruitmentAlreadyJoined,
    BattleRecruitmentNotJoined,

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

    // Recruitment messages
    RecruitmentNormal,
    RecruitmentSixElements,
    RecruitmentMemberFull,
    RecruitmentBefore5Minutes,
    RecruitmentStart,
    RecruitmentEventDateLabel,
    RecruitmentDateFormat,
    RecruitmentDismissalTimesLabel,
    RecruitmentElementFire,
    RecruitmentElementWater,
    RecruitmentElementEarth,
    RecruitmentElementWind,
    RecruitmentElementLight,
    RecruitmentElementDark,
    RecruitmentAllElements,
    RecruitmentNoParticipants,
    RecruitmentLeaveAllButton,

    // Timezone messages (deprecated - use GuildSettings instead)
    TimezoneSetSuccess,
    TimezoneShowCurrent,

    // Guild settings messages
    GuildSettingsSetSuccess,
    GuildSettingsShowSuccess,
    GuildSettingsNotSet,

    // Recruit role messages
    RecruitRoleAddSuccess,
    RecruitRoleRemoveSuccess,

    // Recruit messages
    RecruitCancelAlreadyCancelled,
    RecruitCancelMessageDeleted,
    RecruitCancelInvalidMessage,
    RecruitCancelNotFound,
    RecruitCancelError,
    RecruitCancelledMessageSuffix,
    RecruitCancelNotificationNoParticipants,
    RecruitCancelNotificationWithParticipants,
    RecruitCancellingProgress,
    RecruitChangeNoChanges,
    RecruitChangeSuccess,

    // Recruitment dismissal messages
    RecruitmentDismissalNotification,

    // General messages
    MessagesWelcome,
    MessagesHelp,
}

impl MessageTextId {
    /// MessageIdを文字列キーに変換
    ///
    /// # 戻り値
    /// YAMLファイル内のキー（例: "timezone.show_current"）
    pub fn as_str(&self) -> &'static str {
        match self {
            // Common
            MessageTextId::CommonSuccess => "common.success",
            MessageTextId::CommonError => "common.error",
            MessageTextId::CommonWarning => "common.warning",
            MessageTextId::CommonInfo => "common.info",
            MessageTextId::CommonYes => "common.yes",
            MessageTextId::CommonNo => "common.no",
            MessageTextId::CommonCancel => "common.cancel",
            MessageTextId::CommonConfirm => "common.confirm",
            MessageTextId::CommonLoading => "common.loading",
            MessageTextId::CommonUnknown => "common.unknown",

            // Battle recruitment
            MessageTextId::BattleRecruitmentTitle => "battle_recruitment.title",
            MessageTextId::BattleRecruitmentNewRecruitment => "battle_recruitment.new_recruitment",
            MessageTextId::BattleRecruitmentRecruitmentCancelled => {
                "battle_recruitment.recruitment_cancelled"
            }
            MessageTextId::BattleRecruitmentRecruitmentClosed => {
                "battle_recruitment.recruitment_closed"
            }
            MessageTextId::BattleRecruitmentRecruitmentFull => {
                "battle_recruitment.recruitment_full"
            }
            MessageTextId::BattleRecruitmentJoinSuccess => "battle_recruitment.join_success",
            MessageTextId::BattleRecruitmentLeaveSuccess => "battle_recruitment.leave_success",
            MessageTextId::BattleRecruitmentNotFound => "battle_recruitment.not_found",
            MessageTextId::BattleRecruitmentAlreadyJoined => "battle_recruitment.already_joined",
            MessageTextId::BattleRecruitmentNotJoined => "battle_recruitment.not_joined",

            // Errors
            MessageTextId::ErrorsInvalidInput => "errors.invalid_input",
            MessageTextId::ErrorsPermissionDenied => "errors.permission_denied",
            MessageTextId::ErrorsInternalError => "errors.internal_error",
            MessageTextId::ErrorsUserNotFound => "errors.user_not_found",
            MessageTextId::ErrorsCommandFailed => "errors.command_failed",
            MessageTextId::ErrorsEnvVarNotSet => "errors.env_var_not_set",
            MessageTextId::ErrorsGuildOnly => "errors.guild_only",
            MessageTextId::ErrorsSpreadsheetNotRegistered => "errors.spreadsheet_not_registered",
            MessageTextId::ErrorsSpreadsheetConfigFetchFailed => {
                "errors.spreadsheet_config_fetch_failed"
            }

            // Spreadsheet
            MessageTextId::SpreadsheetLoading => "spreadsheet.loading",
            MessageTextId::SpreadsheetLoadSuccess => "spreadsheet.load_success",
            MessageTextId::SpreadsheetLoadPartialSuccess => "spreadsheet.load_partial_success",
            MessageTextId::SpreadsheetLoadFailed => "spreadsheet.load_failed",
            MessageTextId::SpreadsheetRegistering => "spreadsheet.registering",
            MessageTextId::SpreadsheetRegisterSuccess => "spreadsheet.register_success",
            MessageTextId::SpreadsheetRegisterFailed => "spreadsheet.register_failed",
            MessageTextId::SpreadsheetPushing => "spreadsheet.pushing",
            MessageTextId::SpreadsheetPushSuccess => "spreadsheet.push_success",
            MessageTextId::SpreadsheetPushPartialSuccess => "spreadsheet.push_partial_success",
            MessageTextId::SpreadsheetPushFailed => "spreadsheet.push_failed",
            MessageTextId::SpreadsheetGlobalPushing => "spreadsheet.global_pushing",
            MessageTextId::SpreadsheetGlobalPushSuccess => "spreadsheet.global_push_success",
            MessageTextId::SpreadsheetGlobalPushPartialSuccess => {
                "spreadsheet.global_push_partial_success"
            }
            MessageTextId::SpreadsheetGlobalPushFailed => "spreadsheet.global_push_failed",

            // Kosenjo
            MessageTextId::KosenjoBefore3Days => "kosenjo.before_3_days",
            MessageTextId::KosenjoBefore1Day => "kosenjo.before_1_day",
            MessageTextId::KosenjoQualifyingStart => "kosenjo.qualifying_start",
            MessageTextId::KosenjoQualifyingEnd => "kosenjo.qualifying_end",
            MessageTextId::KosenjoQualifyingEndNoInterval => "kosenjo.qualifying_end_no_interval",
            MessageTextId::KosenjoMainTournamentBefore1Day => {
                "kosenjo.main_tournament_before_1_day"
            }
            MessageTextId::KosenjoMainTournamentDayStart => "kosenjo.main_tournament_day_start",
            MessageTextId::KosenjoMainTournamentHalfDay => "kosenjo.main_tournament_half_day",
            MessageTextId::KosenjoMainTournamentDayEnd => "kosenjo.main_tournament_day_end",
            MessageTextId::KosenjoMainTournamentEnd => "kosenjo.main_tournament_end",
            MessageTextId::KosenjoSpBattleEnd => "kosenjo.sp_battle_end",
            MessageTextId::KosenjoTeamAbility1 => "kosenjo.team_ability_1",
            MessageTextId::KosenjoTeamAbility2 => "kosenjo.team_ability_2",

            // Dorebara
            MessageTextId::DorebaraStart => "dorebara.start",
            MessageTextId::DorebaraEnd => "dorebara.end",
            MessageTextId::DorebaraReset => "dorebara.reset",
            MessageTextId::DorebaraVariant => "dorebara.variant",
            MessageTextId::DorebaraLastDay => "dorebara.last_day",

            // Bot
            MessageTextId::BotMention => "bot.mention",
            MessageTextId::BotMentionSix => "bot.mention_six",
            MessageTextId::BotMentionCalling => "bot.mention_calling",

            // Omikuji
            MessageTextId::OmikujiHihi => "omikuji.hihi",
            MessageTextId::OmikujiHakyoku => "omikuji.hakyoku",
            MessageTextId::OmikujiOmegaUnit => "omikuji.omega_unit",

            // Recruitment
            MessageTextId::RecruitmentNormal => "recruitment.normal",
            MessageTextId::RecruitmentSixElements => "recruitment.six_elements",
            MessageTextId::RecruitmentMemberFull => "recruitment.member_full",
            MessageTextId::RecruitmentBefore5Minutes => "recruitment.before_5_minutes",
            MessageTextId::RecruitmentStart => "recruitment.start",
            MessageTextId::RecruitmentEventDateLabel => "recruitment.event_date_label",
            MessageTextId::RecruitmentDateFormat => "recruitment.date_format",
            MessageTextId::RecruitmentDismissalTimesLabel => "recruitment.dismissal_times_label",
            MessageTextId::RecruitmentElementFire => "recruitment.element_fire",
            MessageTextId::RecruitmentElementWater => "recruitment.element_water",
            MessageTextId::RecruitmentElementEarth => "recruitment.element_earth",
            MessageTextId::RecruitmentElementWind => "recruitment.element_wind",
            MessageTextId::RecruitmentElementLight => "recruitment.element_light",
            MessageTextId::RecruitmentElementDark => "recruitment.element_dark",
            MessageTextId::RecruitmentAllElements => "recruitment.all_elements",
            MessageTextId::RecruitmentNoParticipants => "recruitment.no_participants",
            MessageTextId::RecruitmentLeaveAllButton => "recruitment.leave_all_button",

            // Timezone
            MessageTextId::TimezoneSetSuccess => "timezone.set_success",
            MessageTextId::TimezoneShowCurrent => "timezone.show_current",

            // Guild settings
            MessageTextId::GuildSettingsSetSuccess => "guild_settings.set_success",
            MessageTextId::GuildSettingsShowSuccess => "guild_settings.show_success",
            MessageTextId::GuildSettingsNotSet => "guild_settings.not_set",

            // Recruit role
            MessageTextId::RecruitRoleAddSuccess => "recruit_role.add_success",
            MessageTextId::RecruitRoleRemoveSuccess => "recruit_role.remove_success",

            // Recruit
            MessageTextId::RecruitCancelAlreadyCancelled => "recruit.cancel_already_cancelled",
            MessageTextId::RecruitCancelMessageDeleted => "recruit.cancel_message_deleted",
            MessageTextId::RecruitCancelInvalidMessage => "recruit.cancel_invalid_message",
            MessageTextId::RecruitCancelNotFound => "recruit.cancel_not_found",
            MessageTextId::RecruitCancelError => "recruit.cancel_error",
            MessageTextId::RecruitCancelledMessageSuffix => "recruit.cancelled_message_suffix",
            MessageTextId::RecruitCancelNotificationNoParticipants => {
                "recruit.cancel_notification_no_participants"
            }
            MessageTextId::RecruitCancelNotificationWithParticipants => {
                "recruit.cancel_notification_with_participants"
            }
            MessageTextId::RecruitCancellingProgress => "recruit.cancelling_progress",
            MessageTextId::RecruitChangeNoChanges => "recruit.change_no_changes",
            MessageTextId::RecruitChangeSuccess => "recruit.change_success",

            // Recruitment dismissal
            MessageTextId::RecruitmentDismissalNotification => "recruitment.dismissal_notification",

            // Messages
            MessageTextId::MessagesWelcome => "messages.welcome",
            MessageTextId::MessagesHelp => "messages.help",
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
}
