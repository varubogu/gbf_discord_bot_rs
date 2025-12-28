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

            // Recruitment UI
            MessageTextId::RecruitmentUiTitle => "recruitment.ui.title",
            MessageTextId::RecruitmentUiNewRecruitment => "recruitment.ui.new_recruitment",
            MessageTextId::RecruitmentUiRecruitmentCancelled => {
                "recruitment.ui.recruitment_cancelled"
            }
            MessageTextId::RecruitmentUiRecruitmentClosed => "recruitment.ui.recruitment_closed",
            MessageTextId::RecruitmentUiRecruitmentFull => "recruitment.ui.recruitment_full",
            MessageTextId::RecruitmentUiJoinSuccess => "recruitment.ui.join_success",
            MessageTextId::RecruitmentUiLeaveSuccess => "recruitment.ui.leave_success",
            MessageTextId::RecruitmentUiNotFound => "recruitment.ui.not_found",
            MessageTextId::RecruitmentUiAlreadyJoined => "recruitment.ui.already_joined",
            MessageTextId::RecruitmentUiNotJoined => "recruitment.ui.not_joined",

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

            // Recruitment display
            MessageTextId::RecruitmentDisplayNormal => "recruitment.display.normal",
            MessageTextId::RecruitmentDisplaySixElements => "recruitment.display.six_elements",
            MessageTextId::RecruitmentDisplayEventDateLabel => {
                "recruitment.display.event_date_label"
            }
            MessageTextId::RecruitmentDisplayDateFormat => "recruitment.display.date_format",
            MessageTextId::RecruitmentDisplayDismissalTimesLabel => {
                "recruitment.display.dismissal_times_label"
            }
            MessageTextId::RecruitmentDisplayElementFire => "recruitment.display.element_fire",
            MessageTextId::RecruitmentDisplayElementWater => "recruitment.display.element_water",
            MessageTextId::RecruitmentDisplayElementEarth => "recruitment.display.element_earth",
            MessageTextId::RecruitmentDisplayElementWind => "recruitment.display.element_wind",
            MessageTextId::RecruitmentDisplayElementLight => "recruitment.display.element_light",
            MessageTextId::RecruitmentDisplayElementDark => "recruitment.display.element_dark",
            MessageTextId::RecruitmentDisplayAllElements => "recruitment.display.all_elements",
            MessageTextId::RecruitmentDisplayNoParticipants => {
                "recruitment.display.no_participants"
            }
            MessageTextId::RecruitmentDisplayLeaveAllButton => {
                "recruitment.display.leave_all_button"
            }

            // Recruitment notification
            MessageTextId::RecruitmentNotificationMemberFull => {
                "recruitment.notification.member_full"
            }
            MessageTextId::RecruitmentNotificationBefore5Minutes => {
                "recruitment.notification.before_5_minutes"
            }
            MessageTextId::RecruitmentNotificationStart => "recruitment.notification.start",
            MessageTextId::RecruitmentNotificationDismissal => "recruitment.notification.dismissal",
            MessageTextId::RecruitmentNotificationDismissalWithParticipants => {
                "recruitment.notification.dismissal_with_participants"
            }

            // Timezone
            MessageTextId::TimezoneSetSuccess => "timezone.set_success",
            MessageTextId::TimezoneShowCurrent => "timezone.show_current",

            // Guild settings
            MessageTextId::GuildSettingsSetSuccess => "guild_settings.set_success",
            MessageTextId::GuildSettingsShowSuccess => "guild_settings.show_success",
            MessageTextId::GuildSettingsNotSet => "guild_settings.not_set",

            // Recruitment role
            MessageTextId::RecruitmentRoleAddSuccess => "recruitment.role.add_success",
            MessageTextId::RecruitmentRoleRemoveSuccess => "recruitment.role.remove_success",

            // Recruitment command
            MessageTextId::RecruitmentCommandCancelAlreadyCancelled => {
                "recruitment.command.cancel_already_cancelled"
            }
            MessageTextId::RecruitmentCommandCancelMessageDeleted => {
                "recruitment.command.cancel_message_deleted"
            }
            MessageTextId::RecruitmentCommandCancelInvalidMessage => {
                "recruitment.command.cancel_invalid_message"
            }
            MessageTextId::RecruitmentCommandCancelNotFound => {
                "recruitment.command.cancel_not_found"
            }
            MessageTextId::RecruitmentCommandCancelError => "recruitment.command.cancel_error",
            MessageTextId::RecruitmentCommandCancelledMessageSuffix => {
                "recruitment.command.cancelled_message_suffix"
            }
            MessageTextId::RecruitmentCommandCancelNotificationNoParticipants => {
                "recruitment.command.cancel_notification_no_participants"
            }
            MessageTextId::RecruitmentCommandCancelNotificationWithParticipants => {
                "recruitment.command.cancel_notification_with_participants"
            }
            MessageTextId::RecruitmentCommandCancellingProgress => {
                "recruitment.command.cancelling_progress"
            }
            MessageTextId::RecruitmentCommandChangeNoChanges => {
                "recruitment.command.change_no_changes"
            }
            MessageTextId::RecruitmentCommandChangeSuccess => "recruitment.command.change_success",

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
        let yaml_content = fs::read_to_string(yaml_path)
            .expect("locales/messages.yml が見つかりません");

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
            MessageTextId::RecruitmentCommandCancelError,
            MessageTextId::RecruitmentCommandCancelledMessageSuffix,
            MessageTextId::RecruitmentCommandCancelNotificationNoParticipants,
            MessageTextId::RecruitmentCommandCancelNotificationWithParticipants,
            MessageTextId::RecruitmentCommandCancellingProgress,
            MessageTextId::RecruitmentCommandChangeNoChanges,
            MessageTextId::RecruitmentCommandChangeSuccess,
            MessageTextId::MessagesWelcome,
            MessageTextId::MessagesHelp,
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
