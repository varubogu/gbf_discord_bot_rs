/// メッセージID定義
///
/// メッセージIDを型安全に管理するためのenum
/// 全てのメッセージIDはこのenumで定義される
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageId {
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
    RecruitChangeNoChanges,
    RecruitChangeSuccess,

    // General messages
    MessagesWelcome,
    MessagesHelp,
}

impl MessageId {
    /// MessageIdを文字列キーに変換
    ///
    /// # 戻り値
    /// YAMLファイル内のキー（例: "timezone.show_current"）
    pub fn as_str(&self) -> &'static str {
        match self {
            // Common
            MessageId::CommonSuccess => "common.success",
            MessageId::CommonError => "common.error",
            MessageId::CommonWarning => "common.warning",
            MessageId::CommonInfo => "common.info",
            MessageId::CommonYes => "common.yes",
            MessageId::CommonNo => "common.no",
            MessageId::CommonCancel => "common.cancel",
            MessageId::CommonConfirm => "common.confirm",
            MessageId::CommonLoading => "common.loading",
            MessageId::CommonUnknown => "common.unknown",

            // Battle recruitment
            MessageId::BattleRecruitmentTitle => "battle_recruitment.title",
            MessageId::BattleRecruitmentNewRecruitment => "battle_recruitment.new_recruitment",
            MessageId::BattleRecruitmentRecruitmentCancelled => "battle_recruitment.recruitment_cancelled",
            MessageId::BattleRecruitmentRecruitmentClosed => "battle_recruitment.recruitment_closed",
            MessageId::BattleRecruitmentRecruitmentFull => "battle_recruitment.recruitment_full",
            MessageId::BattleRecruitmentJoinSuccess => "battle_recruitment.join_success",
            MessageId::BattleRecruitmentLeaveSuccess => "battle_recruitment.leave_success",
            MessageId::BattleRecruitmentNotFound => "battle_recruitment.not_found",
            MessageId::BattleRecruitmentAlreadyJoined => "battle_recruitment.already_joined",
            MessageId::BattleRecruitmentNotJoined => "battle_recruitment.not_joined",

            // Errors
            MessageId::ErrorsInvalidInput => "errors.invalid_input",
            MessageId::ErrorsPermissionDenied => "errors.permission_denied",
            MessageId::ErrorsInternalError => "errors.internal_error",
            MessageId::ErrorsUserNotFound => "errors.user_not_found",
            MessageId::ErrorsCommandFailed => "errors.command_failed",
            MessageId::ErrorsEnvVarNotSet => "errors.env_var_not_set",
            MessageId::ErrorsGuildOnly => "errors.guild_only",
            MessageId::ErrorsSpreadsheetNotRegistered => "errors.spreadsheet_not_registered",
            MessageId::ErrorsSpreadsheetConfigFetchFailed => "errors.spreadsheet_config_fetch_failed",

            // Spreadsheet
            MessageId::SpreadsheetLoading => "spreadsheet.loading",
            MessageId::SpreadsheetLoadSuccess => "spreadsheet.load_success",
            MessageId::SpreadsheetLoadPartialSuccess => "spreadsheet.load_partial_success",
            MessageId::SpreadsheetLoadFailed => "spreadsheet.load_failed",
            MessageId::SpreadsheetRegistering => "spreadsheet.registering",
            MessageId::SpreadsheetRegisterSuccess => "spreadsheet.register_success",
            MessageId::SpreadsheetRegisterFailed => "spreadsheet.register_failed",
            MessageId::SpreadsheetPushing => "spreadsheet.pushing",
            MessageId::SpreadsheetPushSuccess => "spreadsheet.push_success",
            MessageId::SpreadsheetPushPartialSuccess => "spreadsheet.push_partial_success",
            MessageId::SpreadsheetPushFailed => "spreadsheet.push_failed",
            MessageId::SpreadsheetGlobalPushing => "spreadsheet.global_pushing",
            MessageId::SpreadsheetGlobalPushSuccess => "spreadsheet.global_push_success",
            MessageId::SpreadsheetGlobalPushPartialSuccess => "spreadsheet.global_push_partial_success",
            MessageId::SpreadsheetGlobalPushFailed => "spreadsheet.global_push_failed",

            // Kosenjo
            MessageId::KosenjoBefore3Days => "kosenjo.before_3_days",
            MessageId::KosenjoBefore1Day => "kosenjo.before_1_day",
            MessageId::KosenjoQualifyingStart => "kosenjo.qualifying_start",
            MessageId::KosenjoQualifyingEnd => "kosenjo.qualifying_end",
            MessageId::KosenjoQualifyingEndNoInterval => "kosenjo.qualifying_end_no_interval",
            MessageId::KosenjoMainTournamentBefore1Day => "kosenjo.main_tournament_before_1_day",
            MessageId::KosenjoMainTournamentDayStart => "kosenjo.main_tournament_day_start",
            MessageId::KosenjoMainTournamentHalfDay => "kosenjo.main_tournament_half_day",
            MessageId::KosenjoMainTournamentDayEnd => "kosenjo.main_tournament_day_end",
            MessageId::KosenjoMainTournamentEnd => "kosenjo.main_tournament_end",
            MessageId::KosenjoSpBattleEnd => "kosenjo.sp_battle_end",
            MessageId::KosenjoTeamAbility1 => "kosenjo.team_ability_1",
            MessageId::KosenjoTeamAbility2 => "kosenjo.team_ability_2",

            // Dorebara
            MessageId::DorebaraStart => "dorebara.start",
            MessageId::DorebaraEnd => "dorebara.end",
            MessageId::DorebaraReset => "dorebara.reset",
            MessageId::DorebaraVariant => "dorebara.variant",
            MessageId::DorebaraLastDay => "dorebara.last_day",

            // Bot
            MessageId::BotMention => "bot.mention",
            MessageId::BotMentionSix => "bot.mention_six",
            MessageId::BotMentionCalling => "bot.mention_calling",

            // Omikuji
            MessageId::OmikujiHihi => "omikuji.hihi",
            MessageId::OmikujiHakyoku => "omikuji.hakyoku",
            MessageId::OmikujiOmegaUnit => "omikuji.omega_unit",

            // Recruitment
            MessageId::RecruitmentNormal => "recruitment.normal",
            MessageId::RecruitmentSixElements => "recruitment.six_elements",
            MessageId::RecruitmentMemberFull => "recruitment.member_full",
            MessageId::RecruitmentBefore5Minutes => "recruitment.before_5_minutes",
            MessageId::RecruitmentStart => "recruitment.start",
            MessageId::RecruitmentEventDateLabel => "recruitment.event_date_label",
            MessageId::RecruitmentDateFormat => "recruitment.date_format",
            MessageId::RecruitmentElementFire => "recruitment.element_fire",
            MessageId::RecruitmentElementWater => "recruitment.element_water",
            MessageId::RecruitmentElementEarth => "recruitment.element_earth",
            MessageId::RecruitmentElementWind => "recruitment.element_wind",
            MessageId::RecruitmentElementLight => "recruitment.element_light",
            MessageId::RecruitmentElementDark => "recruitment.element_dark",
            MessageId::RecruitmentAllElements => "recruitment.all_elements",
            MessageId::RecruitmentNoParticipants => "recruitment.no_participants",
            MessageId::RecruitmentLeaveAllButton => "recruitment.leave_all_button",

            // Timezone
            MessageId::TimezoneSetSuccess => "timezone.set_success",
            MessageId::TimezoneShowCurrent => "timezone.show_current",

            // Guild settings
            MessageId::GuildSettingsSetSuccess => "guild_settings.set_success",
            MessageId::GuildSettingsShowSuccess => "guild_settings.show_success",
            MessageId::GuildSettingsNotSet => "guild_settings.not_set",

            // Recruit role
            MessageId::RecruitRoleAddSuccess => "recruit_role.add_success",
            MessageId::RecruitRoleRemoveSuccess => "recruit_role.remove_success",

            // Recruit
            MessageId::RecruitCancelAlreadyCancelled => "recruit.cancel_already_cancelled",
            MessageId::RecruitCancelMessageDeleted => "recruit.cancel_message_deleted",
            MessageId::RecruitCancelInvalidMessage => "recruit.cancel_invalid_message",
            MessageId::RecruitCancelNotFound => "recruit.cancel_not_found",
            MessageId::RecruitCancelError => "recruit.cancel_error",
            MessageId::RecruitChangeNoChanges => "recruit.change_no_changes",
            MessageId::RecruitChangeSuccess => "recruit.change_success",

            // Messages
            MessageId::MessagesWelcome => "messages.welcome",
            MessageId::MessagesHelp => "messages.help",
        }
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_as_str() {
        assert_eq!(MessageId::TimezoneShowCurrent.as_str(), "timezone.show_current");
        assert_eq!(MessageId::TimezoneSetSuccess.as_str(), "timezone.set_success");
        assert_eq!(MessageId::ErrorsGuildOnly.as_str(), "errors.guild_only");
    }

    #[test]
    fn test_message_id_display() {
        assert_eq!(MessageId::TimezoneShowCurrent.to_string(), "timezone.show_current");
    }
}
