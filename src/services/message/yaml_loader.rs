/// YAML メッセージローダー
///
/// rust-i18n の t! マクロを使用して、メッセージIDに対応する翻訳を取得します。
/// コンパイル時に最適化され、実行時のオーバーヘッドはほぼゼロです。
use crate::services::message::message_text_id::keys;
use rust_i18n::t;

/// メッセージIDとロケールからYAMLメッセージを取得
///
/// # 引数
/// * `message_id` - メッセージID（例: "timezone.show_current"）
/// * `locale` - ロケール（"ja" または "en"）
///
/// # 戻り値
/// メッセージが見つかった場合は Some(String)、見つからない場合は None
///
/// # 実装ノート
/// このコードは意図的にシンプルな設計になっています。
/// rust-i18n の t! マクロはコンパイル時に文字列リテラルを要求するため、
/// 動的な message_id を直接渡すことができません。
/// そのため、各メッセージIDごとに個別にマッチングする必要があります。
///
/// メッセージキーの定義は `message_text_id::keys` モジュールで一元管理されています。
/// これにより、`as_str()` と `yaml_loader.rs` で同じ定数を参照でき、
/// 単一の情報源(Single Source of Truth)を実現しています。
pub fn get_yaml_message(message_id: &str, locale: &str) -> Option<String> {
    // t! マクロはコンパイル時に評価されるため、動的な文字列を渡すことができない
    // そのため、各メッセージIDを個別にマッチングする必要がある
    match message_id {
        // Common messages
        keys::COMMON_SUCCESS => Some(t!(keys::COMMON_SUCCESS, locale = locale).to_string()),
        keys::COMMON_ERROR => Some(t!(keys::COMMON_ERROR, locale = locale).to_string()),
        keys::COMMON_ERROR_PREFIX => {
            Some(t!(keys::COMMON_ERROR_PREFIX, locale = locale).to_string())
        }
        keys::COMMON_WARNING => Some(t!(keys::COMMON_WARNING, locale = locale).to_string()),
        keys::COMMON_INFO => Some(t!(keys::COMMON_INFO, locale = locale).to_string()),
        keys::COMMON_YES => Some(t!(keys::COMMON_YES, locale = locale).to_string()),
        keys::COMMON_NO => Some(t!(keys::COMMON_NO, locale = locale).to_string()),
        keys::COMMON_CANCEL => Some(t!(keys::COMMON_CANCEL, locale = locale).to_string()),
        keys::COMMON_CONFIRM => Some(t!(keys::COMMON_CONFIRM, locale = locale).to_string()),
        keys::COMMON_LOADING => Some(t!(keys::COMMON_LOADING, locale = locale).to_string()),
        keys::COMMON_UNKNOWN => Some(t!(keys::COMMON_UNKNOWN, locale = locale).to_string()),

        // Recruitment UI messages
        keys::RECRUITMENT_UI_TITLE => {
            Some(t!(keys::RECRUITMENT_UI_TITLE, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_NEW_RECRUITMENT => {
            Some(t!(keys::RECRUITMENT_UI_NEW_RECRUITMENT, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_RECRUITMENT_CANCELLED => {
            Some(t!(keys::RECRUITMENT_UI_RECRUITMENT_CANCELLED, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_RECRUITMENT_CLOSED => {
            Some(t!(keys::RECRUITMENT_UI_RECRUITMENT_CLOSED, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_RECRUITMENT_FULL => {
            Some(t!(keys::RECRUITMENT_UI_RECRUITMENT_FULL, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_JOIN_SUCCESS => {
            Some(t!(keys::RECRUITMENT_UI_JOIN_SUCCESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_LEAVE_SUCCESS => {
            Some(t!(keys::RECRUITMENT_UI_LEAVE_SUCCESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_NOT_FOUND => {
            Some(t!(keys::RECRUITMENT_UI_NOT_FOUND, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_ALREADY_JOINED => {
            Some(t!(keys::RECRUITMENT_UI_ALREADY_JOINED, locale = locale).to_string())
        }
        keys::RECRUITMENT_UI_NOT_JOINED => {
            Some(t!(keys::RECRUITMENT_UI_NOT_JOINED, locale = locale).to_string())
        }

        // Error messages
        keys::ERRORS_INVALID_INPUT => {
            Some(t!(keys::ERRORS_INVALID_INPUT, locale = locale).to_string())
        }
        keys::ERRORS_PERMISSION_DENIED => {
            Some(t!(keys::ERRORS_PERMISSION_DENIED, locale = locale).to_string())
        }
        keys::ERRORS_INTERNAL_ERROR => {
            Some(t!(keys::ERRORS_INTERNAL_ERROR, locale = locale).to_string())
        }
        keys::ERRORS_USER_NOT_FOUND => {
            Some(t!(keys::ERRORS_USER_NOT_FOUND, locale = locale).to_string())
        }
        keys::ERRORS_COMMAND_FAILED => {
            Some(t!(keys::ERRORS_COMMAND_FAILED, locale = locale).to_string())
        }
        keys::ERRORS_ENV_VAR_NOT_SET => {
            Some(t!(keys::ERRORS_ENV_VAR_NOT_SET, locale = locale).to_string())
        }
        keys::ERRORS_GUILD_ONLY => Some(t!(keys::ERRORS_GUILD_ONLY, locale = locale).to_string()),
        keys::ERRORS_SPREADSHEET_NOT_REGISTERED => {
            Some(t!(keys::ERRORS_SPREADSHEET_NOT_REGISTERED, locale = locale).to_string())
        }
        keys::ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED => Some(
            t!(
                keys::ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED,
                locale = locale
            )
            .to_string(),
        ),

        // Spreadsheet messages
        keys::SPREADSHEET_LOADING => {
            Some(t!(keys::SPREADSHEET_LOADING, locale = locale).to_string())
        }
        keys::SPREADSHEET_LOAD_SUCCESS => {
            Some(t!(keys::SPREADSHEET_LOAD_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_LOAD_PARTIAL_SUCCESS => {
            Some(t!(keys::SPREADSHEET_LOAD_PARTIAL_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_LOAD_FAILED => {
            Some(t!(keys::SPREADSHEET_LOAD_FAILED, locale = locale).to_string())
        }
        keys::SPREADSHEET_REGISTERING => {
            Some(t!(keys::SPREADSHEET_REGISTERING, locale = locale).to_string())
        }
        keys::SPREADSHEET_REGISTER_SUCCESS => {
            Some(t!(keys::SPREADSHEET_REGISTER_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_REGISTER_FAILED => {
            Some(t!(keys::SPREADSHEET_REGISTER_FAILED, locale = locale).to_string())
        }
        keys::SPREADSHEET_PUSHING => {
            Some(t!(keys::SPREADSHEET_PUSHING, locale = locale).to_string())
        }
        keys::SPREADSHEET_PUSH_SUCCESS => {
            Some(t!(keys::SPREADSHEET_PUSH_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_PUSH_PARTIAL_SUCCESS => {
            Some(t!(keys::SPREADSHEET_PUSH_PARTIAL_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_PUSH_FAILED => {
            Some(t!(keys::SPREADSHEET_PUSH_FAILED, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_PUSHING => {
            Some(t!(keys::SPREADSHEET_GLOBAL_PUSHING, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_PUSH_SUCCESS => {
            Some(t!(keys::SPREADSHEET_GLOBAL_PUSH_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS => Some(
            t!(
                keys::SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS,
                locale = locale
            )
            .to_string(),
        ),
        keys::SPREADSHEET_GLOBAL_PUSH_FAILED => {
            Some(t!(keys::SPREADSHEET_GLOBAL_PUSH_FAILED, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_LOADING => {
            Some(t!(keys::SPREADSHEET_GLOBAL_LOADING, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_LOAD_SUCCESS => {
            Some(t!(keys::SPREADSHEET_GLOBAL_LOAD_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_LOAD_FAILED => {
            Some(t!(keys::SPREADSHEET_GLOBAL_LOAD_FAILED, locale = locale).to_string())
        }

        // Kosenjo messages
        keys::KOSENJO_BEFORE_3_DAYS => {
            Some(t!(keys::KOSENJO_BEFORE_3_DAYS, locale = locale).to_string())
        }
        keys::KOSENJO_BEFORE_1_DAY => {
            Some(t!(keys::KOSENJO_BEFORE_1_DAY, locale = locale).to_string())
        }
        keys::KOSENJO_QUALIFYING_START => {
            Some(t!(keys::KOSENJO_QUALIFYING_START, locale = locale).to_string())
        }
        keys::KOSENJO_QUALIFYING_END => {
            Some(t!(keys::KOSENJO_QUALIFYING_END, locale = locale).to_string())
        }
        keys::KOSENJO_QUALIFYING_END_NO_INTERVAL => {
            Some(t!(keys::KOSENJO_QUALIFYING_END_NO_INTERVAL, locale = locale).to_string())
        }
        keys::KOSENJO_MAIN_TOURNAMENT_BEFORE_1_DAY => {
            Some(t!(keys::KOSENJO_MAIN_TOURNAMENT_BEFORE_1_DAY, locale = locale).to_string())
        }
        keys::KOSENJO_MAIN_TOURNAMENT_DAY_START => {
            Some(t!(keys::KOSENJO_MAIN_TOURNAMENT_DAY_START, locale = locale).to_string())
        }
        keys::KOSENJO_MAIN_TOURNAMENT_HALF_DAY => {
            Some(t!(keys::KOSENJO_MAIN_TOURNAMENT_HALF_DAY, locale = locale).to_string())
        }
        keys::KOSENJO_MAIN_TOURNAMENT_DAY_END => {
            Some(t!(keys::KOSENJO_MAIN_TOURNAMENT_DAY_END, locale = locale).to_string())
        }
        keys::KOSENJO_MAIN_TOURNAMENT_END => {
            Some(t!(keys::KOSENJO_MAIN_TOURNAMENT_END, locale = locale).to_string())
        }
        keys::KOSENJO_SP_BATTLE_END => {
            Some(t!(keys::KOSENJO_SP_BATTLE_END, locale = locale).to_string())
        }
        keys::KOSENJO_TEAM_ABILITY_1 => {
            Some(t!(keys::KOSENJO_TEAM_ABILITY_1, locale = locale).to_string())
        }
        keys::KOSENJO_TEAM_ABILITY_2 => {
            Some(t!(keys::KOSENJO_TEAM_ABILITY_2, locale = locale).to_string())
        }

        // Dorebara messages
        keys::DOREBARA_START => Some(t!(keys::DOREBARA_START, locale = locale).to_string()),
        keys::DOREBARA_END => Some(t!(keys::DOREBARA_END, locale = locale).to_string()),
        keys::DOREBARA_RESET => Some(t!(keys::DOREBARA_RESET, locale = locale).to_string()),
        keys::DOREBARA_VARIANT => Some(t!(keys::DOREBARA_VARIANT, locale = locale).to_string()),
        keys::DOREBARA_LAST_DAY => Some(t!(keys::DOREBARA_LAST_DAY, locale = locale).to_string()),

        // Bot messages
        keys::BOT_MENTION => Some(t!(keys::BOT_MENTION, locale = locale).to_string()),
        keys::BOT_MENTION_SIX => Some(t!(keys::BOT_MENTION_SIX, locale = locale).to_string()),
        keys::BOT_MENTION_CALLING => {
            Some(t!(keys::BOT_MENTION_CALLING, locale = locale).to_string())
        }

        // Omikuji messages
        keys::OMIKUJI_HIHI => Some(t!(keys::OMIKUJI_HIHI, locale = locale).to_string()),
        keys::OMIKUJI_HAKYOKU => Some(t!(keys::OMIKUJI_HAKYOKU, locale = locale).to_string()),
        keys::OMIKUJI_OMEGA_UNIT => Some(t!(keys::OMIKUJI_OMEGA_UNIT, locale = locale).to_string()),

        // Recruitment display messages
        keys::RECRUITMENT_DISPLAY_NORMAL => {
            Some(t!(keys::RECRUITMENT_DISPLAY_NORMAL, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_SIX_ELEMENTS => {
            Some(t!(keys::RECRUITMENT_DISPLAY_SIX_ELEMENTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_EVENT_DATE_LABEL => {
            Some(t!(keys::RECRUITMENT_DISPLAY_EVENT_DATE_LABEL, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_DATE_FORMAT => {
            Some(t!(keys::RECRUITMENT_DISPLAY_DATE_FORMAT, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL => Some(
            t!(
                keys::RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_DISPLAY_ELEMENT_FIRE => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_FIRE, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ELEMENT_WATER => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_WATER, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ELEMENT_EARTH => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_EARTH, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ELEMENT_WIND => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_WIND, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ELEMENT_LIGHT => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_LIGHT, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ELEMENT_DARK => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ELEMENT_DARK, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_ALL_ELEMENTS => {
            Some(t!(keys::RECRUITMENT_DISPLAY_ALL_ELEMENTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_NO_PARTICIPANTS => {
            Some(t!(keys::RECRUITMENT_DISPLAY_NO_PARTICIPANTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_LEAVE_ALL_BUTTON => {
            Some(t!(keys::RECRUITMENT_DISPLAY_LEAVE_ALL_BUTTON, locale = locale).to_string())
        }

        // Recruitment notification messages
        keys::RECRUITMENT_NOTIFICATION_MEMBER_FULL => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_MEMBER_FULL, locale = locale).to_string())
        }
        keys::RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES => Some(
            t!(
                keys::RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_NOTIFICATION_START => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_START, locale = locale).to_string())
        }
        keys::RECRUITMENT_NOTIFICATION_DISMISSAL => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_DISMISSAL, locale = locale).to_string())
        }
        keys::RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS => Some(
            t!(
                keys::RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS,
                locale = locale
            )
            .to_string(),
        ),

        // Timezone messages
        keys::TIMEZONE_SET_SUCCESS => {
            Some(t!(keys::TIMEZONE_SET_SUCCESS, locale = locale).to_string())
        }
        keys::TIMEZONE_SHOW_CURRENT => {
            Some(t!(keys::TIMEZONE_SHOW_CURRENT, locale = locale).to_string())
        }

        // Guild settings messages
        keys::GUILD_SETTINGS_SET_SUCCESS => {
            Some(t!(keys::GUILD_SETTINGS_SET_SUCCESS, locale = locale).to_string())
        }
        keys::GUILD_SETTINGS_SHOW_SUCCESS => {
            Some(t!(keys::GUILD_SETTINGS_SHOW_SUCCESS, locale = locale).to_string())
        }
        keys::GUILD_SETTINGS_NOT_SET => {
            Some(t!(keys::GUILD_SETTINGS_NOT_SET, locale = locale).to_string())
        }

        // Recruitment role messages
        keys::RECRUITMENT_ROLE_ADD_SUCCESS => {
            Some(t!(keys::RECRUITMENT_ROLE_ADD_SUCCESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_REMOVE_SUCCESS => {
            Some(t!(keys::RECRUITMENT_ROLE_REMOVE_SUCCESS, locale = locale).to_string())
        }

        // Recruitment command messages
        keys::RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_NOT_FOUND => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_NOT_FOUND, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_ERROR => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_ERROR, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCELLING_PROGRESS => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCELLING_PROGRESS,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_NO_CHANGES => {
            Some(t!(keys::RECRUITMENT_COMMAND_CHANGE_NO_CHANGES, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CHANGE_SUCCESS => {
            Some(t!(keys::RECRUITMENT_COMMAND_CHANGE_SUCCESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_CONFIRM_PROMPT => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_CONFIRM_PROMPT,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_ABORTED => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_ABORTED, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_UNKNOWN_SELECTION => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CANCEL_UNKNOWN_SELECTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CANCEL_TIMEOUT => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_TIMEOUT, locale = locale).to_string())
        }

        // General messages
        keys::MESSAGES_WELCOME => Some(t!(keys::MESSAGES_WELCOME, locale = locale).to_string()),
        keys::MESSAGES_HELP => Some(t!(keys::MESSAGES_HELP, locale = locale).to_string()),

        // Auto recruitment messages
        keys::AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED => Some(
            t!(
                keys::AUTO_RECRUITMENT_CHANNEL_CREATE_FAILED,
                locale = locale
            )
            .to_string(),
        ),
        keys::AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER => Some(
            t!(
                keys::AUTO_RECRUITMENT_TIME_SELECT_PLACEHOLDER,
                locale = locale
            )
            .to_string(),
        ),
        keys::AUTO_RECRUITMENT_UNREGISTER_IN_CATEGORY_ERROR => Some(
            t!(
                keys::AUTO_RECRUITMENT_UNREGISTER_IN_CATEGORY_ERROR,
                locale = locale
            )
            .to_string(),
        ),

        // App error messages
        keys::APP_ERROR_DATABASE => Some(t!(keys::APP_ERROR_DATABASE, locale = locale).to_string()),
        keys::APP_ERROR_DISCORD => Some(t!(keys::APP_ERROR_DISCORD, locale = locale).to_string()),
        keys::APP_ERROR_CONFIG => Some(t!(keys::APP_ERROR_CONFIG, locale = locale).to_string()),
        keys::APP_ERROR_VALIDATION => {
            Some(t!(keys::APP_ERROR_VALIDATION, locale = locale).to_string())
        }
        keys::APP_ERROR_DISCORD_OPERATION => {
            Some(t!(keys::APP_ERROR_DISCORD_OPERATION, locale = locale).to_string())
        }
        keys::APP_ERROR_CHANNEL_CREATION_FAILED => {
            Some(t!(keys::APP_ERROR_CHANNEL_CREATION_FAILED, locale = locale).to_string())
        }
        keys::APP_ERROR_IN_CATEGORY_CHANNEL => {
            Some(t!(keys::APP_ERROR_IN_CATEGORY_CHANNEL, locale = locale).to_string())
        }

        // Schedule command messages
        keys::SCHEDULE_COMMAND_GENERATE_LOADING => {
            Some(t!(keys::SCHEDULE_COMMAND_GENERATE_LOADING, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_TITLE => Some(
            t!(
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FIELD_NAME => Some(
            t!(
                keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FIELD_NAME,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_FIELD_VALUE => Some(
            t!(
                keys::SCHEDULE_COMMAND_GENERATE_SUCCESS_FIELD_VALUE,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FOOTER => Some(
            t!(
                keys::SCHEDULE_COMMAND_SHARED_SUCCESS_FOOTER,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GENERATE_ERROR_TITLE => {
            Some(t!(keys::SCHEDULE_COMMAND_GENERATE_ERROR_TITLE, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_GENERATE_ERROR_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_GENERATE_ERROR_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_SHARED_ERROR_FOOTER => {
            Some(t!(keys::SCHEDULE_COMMAND_SHARED_ERROR_FOOTER, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_LOADING => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_LOADING,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_TITLE => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_FIELD_VALUE => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_SUCCESS_FIELD_VALUE,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_TITLE => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_GLOBAL_GENERATE_ERROR_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_LIST_TITLE => {
            Some(t!(keys::SCHEDULE_COMMAND_LIST_TITLE, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_LIST_EMPTY_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_LIST_EMPTY_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_LIST_FOOTER => {
            Some(t!(keys::SCHEDULE_COMMAND_LIST_FOOTER, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_HISTORY_TITLE => {
            Some(t!(keys::SCHEDULE_COMMAND_HISTORY_TITLE, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_HISTORY_EMPTY_DESCRIPTION => Some(
            t!(
                keys::SCHEDULE_COMMAND_HISTORY_EMPTY_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_HISTORY_TITLE_WITH_DAYS => Some(
            t!(
                keys::SCHEDULE_COMMAND_HISTORY_TITLE_WITH_DAYS,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_HISTORY_FOOTER => {
            Some(t!(keys::SCHEDULE_COMMAND_HISTORY_FOOTER, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_STATS_TITLE_WITH_DAYS => Some(
            t!(
                keys::SCHEDULE_COMMAND_STATS_TITLE_WITH_DAYS,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_STATS_FOOTER => {
            Some(t!(keys::SCHEDULE_COMMAND_STATS_FOOTER, locale = locale).to_string())
        }
        keys::SCHEDULE_COMMAND_STATS_DESCRIPTION_HEADER => Some(
            t!(
                keys::SCHEDULE_COMMAND_STATS_DESCRIPTION_HEADER,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_STATS_MESSAGE_TYPE_HEADER => Some(
            t!(
                keys::SCHEDULE_COMMAND_STATS_MESSAGE_TYPE_HEADER,
                locale = locale
            )
            .to_string(),
        ),
        keys::SCHEDULE_COMMAND_STATS_OTHER_TYPES => {
            Some(t!(keys::SCHEDULE_COMMAND_STATS_OTHER_TYPES, locale = locale).to_string())
        }

        // Recruitment schedule command messages
        keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_ALL => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_ALL, locale = locale).to_string())
        }
        keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_SELF => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_EMPTY_SELF, locale = locale).to_string())
        }
        keys::RECRUITMENT_SCHEDULE_LIST_TITLE_ALL => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_TITLE_ALL, locale = locale).to_string())
        }
        keys::RECRUITMENT_SCHEDULE_LIST_TITLE_SELF => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_TITLE_SELF, locale = locale).to_string())
        }
        keys::RECRUITMENT_SCHEDULE_LIST_STATUS_ENABLED => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_LIST_STATUS_ENABLED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_LIST_STATUS_DISABLED => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_LIST_STATUS_DISABLED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_LIST_DISMISSAL_PREFIX => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_LIST_DISMISSAL_PREFIX,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_LIST_MORE_COUNT => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_MORE_COUNT, locale = locale).to_string())
        }
        keys::RECRUITMENT_SCHEDULE_LIST_FOOTER => {
            Some(t!(keys::RECRUITMENT_SCHEDULE_LIST_FOOTER, locale = locale).to_string())
        }

        // Recruitment list command messages
        keys::RECRUITMENT_LIST_TITLE => {
            Some(t!(keys::RECRUITMENT_LIST_TITLE, locale = locale).to_string())
        }
        keys::RECRUITMENT_LIST_EMPTY => {
            Some(t!(keys::RECRUITMENT_LIST_EMPTY, locale = locale).to_string())
        }
        keys::RECRUITMENT_LIST_LINK_TEXT => {
            Some(t!(keys::RECRUITMENT_LIST_LINK_TEXT, locale = locale).to_string())
        }
        keys::RECRUITMENT_LIST_MORE_COUNT => {
            Some(t!(keys::RECRUITMENT_LIST_MORE_COUNT, locale = locale).to_string())
        }
        keys::RECRUITMENT_LIST_FOOTER => {
            Some(t!(keys::RECRUITMENT_LIST_FOOTER, locale = locale).to_string())
        }

        keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_TITLE => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_DESCRIPTION => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_TOGGLE_SUCCESS_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_TITLE => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_DESCRIPTION => Some(
            t!(
                keys::RECRUITMENT_SCHEDULE_DELETE_SUCCESS_DESCRIPTION,
                locale = locale
            )
            .to_string(),
        ),

        // Recruitment role show messages
        keys::RECRUITMENT_ROLE_SHOW_NOT_REGISTERED => {
            Some(t!(keys::RECRUITMENT_ROLE_SHOW_NOT_REGISTERED, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_SHOW_HEADER => {
            Some(t!(keys::RECRUITMENT_ROLE_SHOW_HEADER, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_SHOW_SECTION_ALL => {
            Some(t!(keys::RECRUITMENT_ROLE_SHOW_SECTION_ALL, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_SHOW_SECTION_QUEST => {
            Some(t!(keys::RECRUITMENT_ROLE_SHOW_SECTION_QUEST, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_SHOW_UNKNOWN_QUEST => {
            Some(t!(keys::RECRUITMENT_ROLE_SHOW_UNKNOWN_QUEST, locale = locale).to_string())
        }

        // Recruitment change panel messages
        keys::RECRUITMENT_COMMAND_CHANGE_PANEL_UNCHANGED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_PANEL_UNCHANGED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_PANEL_CONTENT => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_PANEL_CONTENT,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_OPTION_QUEST_UNCHANGED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_OPTION_QUEST_UNCHANGED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_OPTION_STYLE_UNCHANGED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_OPTION_STYLE_UNCHANGED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_QUEST => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_QUEST,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_STYLE => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_PLACEHOLDER_STYLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_OPEN_DATE => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_OPEN_DATE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_CLEAR_DATE => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_CLEAR_DATE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_APPLY => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_BUTTON_APPLY,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_MODAL_TITLE => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_TITLE,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_LABEL => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_LABEL,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_PLACEHOLDER => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_EVENT_DATE_PLACEHOLDER,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_MODAL_ABSOLUTE_DATETIME_REQUIRED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_ABSOLUTE_DATETIME_REQUIRED,
                locale = locale
            )
            .to_string(),
        ),
        keys::RECRUITMENT_COMMAND_CHANGE_MODAL_PARSE_FAILED => Some(
            t!(
                keys::RECRUITMENT_COMMAND_CHANGE_MODAL_PARSE_FAILED,
                locale = locale
            )
            .to_string(),
        ),

        // Channel show messages
        keys::CHANNEL_SHOW_EMPTY => Some(t!(keys::CHANNEL_SHOW_EMPTY, locale = locale).to_string()),
        keys::CHANNEL_SHOW_HEADER => {
            Some(t!(keys::CHANNEL_SHOW_HEADER, locale = locale).to_string())
        }
        keys::CHANNEL_SHOW_UNSET => Some(t!(keys::CHANNEL_SHOW_UNSET, locale = locale).to_string()),

        // Auto recruitment interaction messages
        keys::AUTO_RECRUITMENT_QUEST_SELECT_REQUIRED => Some(
            t!(
                keys::AUTO_RECRUITMENT_QUEST_SELECT_REQUIRED,
                locale = locale
            )
            .to_string(),
        ),
        keys::AUTO_RECRUITMENT_QUEST_SELECT_REGISTERED => Some(
            t!(
                keys::AUTO_RECRUITMENT_QUEST_SELECT_REGISTERED,
                locale = locale
            )
            .to_string(),
        ),
        keys::AUTO_RECRUITMENT_TIME_SELECT_REQUIRED => {
            Some(t!(keys::AUTO_RECRUITMENT_TIME_SELECT_REQUIRED, locale = locale).to_string())
        }
        keys::AUTO_RECRUITMENT_TIME_SELECT_REGISTERED => Some(
            t!(
                keys::AUTO_RECRUITMENT_TIME_SELECT_REGISTERED,
                locale = locale
            )
            .to_string(),
        ),
        // Help embed messages
        keys::HELP_EMBED_TITLE => Some(t!(keys::HELP_EMBED_TITLE, locale = locale).to_string()),
        keys::HELP_EMBED_DESCRIPTION => {
            Some(t!(keys::HELP_EMBED_DESCRIPTION, locale = locale).to_string())
        }
        keys::HELP_EMBED_COMMANDS_FIELD_TITLE => {
            Some(t!(keys::HELP_EMBED_COMMANDS_FIELD_TITLE, locale = locale).to_string())
        }
        keys::HELP_EMBED_COMMANDS_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_COMMANDS_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_RECRUIT_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_RECRUIT_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_ENVIRON_LOAD_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_ENVIRON_LOAD_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_GSPREAD_LOAD_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_GSPREAD_LOAD_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_GSPREAD_PUSH_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_GSPREAD_PUSH_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_HELP_FIELD_VALUE => {
            Some(t!(keys::HELP_EMBED_HELP_FIELD_VALUE, locale = locale).to_string())
        }
        keys::HELP_EMBED_FOOTER => Some(t!(keys::HELP_EMBED_FOOTER, locale = locale).to_string()),

        // 未知のメッセージID
        _ => None,
    }
}
