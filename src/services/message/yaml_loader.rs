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
        keys::ERRORS_INVALID_INPUT => Some(t!(keys::ERRORS_INVALID_INPUT, locale = locale).to_string()),
        keys::ERRORS_PERMISSION_DENIED => {
            Some(t!(keys::ERRORS_PERMISSION_DENIED, locale = locale).to_string())
        }
        keys::ERRORS_INTERNAL_ERROR => Some(t!(keys::ERRORS_INTERNAL_ERROR, locale = locale).to_string()),
        keys::ERRORS_USER_NOT_FOUND => Some(t!(keys::ERRORS_USER_NOT_FOUND, locale = locale).to_string()),
        keys::ERRORS_COMMAND_FAILED => Some(t!(keys::ERRORS_COMMAND_FAILED, locale = locale).to_string()),
        keys::ERRORS_ENV_VAR_NOT_SET => Some(t!(keys::ERRORS_ENV_VAR_NOT_SET, locale = locale).to_string()),
        keys::ERRORS_GUILD_ONLY => Some(t!(keys::ERRORS_GUILD_ONLY, locale = locale).to_string()),
        keys::ERRORS_SPREADSHEET_NOT_REGISTERED => {
            Some(t!(keys::ERRORS_SPREADSHEET_NOT_REGISTERED, locale = locale).to_string())
        }
        keys::ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED => {
            Some(t!(keys::ERRORS_SPREADSHEET_CONFIG_FETCH_FAILED, locale = locale).to_string())
        }

        // Spreadsheet messages
        keys::SPREADSHEET_LOADING => Some(t!(keys::SPREADSHEET_LOADING, locale = locale).to_string()),
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
        keys::SPREADSHEET_PUSHING => Some(t!(keys::SPREADSHEET_PUSHING, locale = locale).to_string()),
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
        keys::SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS => {
            Some(t!(keys::SPREADSHEET_GLOBAL_PUSH_PARTIAL_SUCCESS, locale = locale).to_string())
        }
        keys::SPREADSHEET_GLOBAL_PUSH_FAILED => {
            Some(t!(keys::SPREADSHEET_GLOBAL_PUSH_FAILED, locale = locale).to_string())
        }

        // Kosenjo messages
        keys::KOSENJO_BEFORE_3_DAYS => Some(t!(keys::KOSENJO_BEFORE_3_DAYS, locale = locale).to_string()),
        keys::KOSENJO_BEFORE_1_DAY => Some(t!(keys::KOSENJO_BEFORE_1_DAY, locale = locale).to_string()),
        keys::KOSENJO_QUALIFYING_START => {
            Some(t!(keys::KOSENJO_QUALIFYING_START, locale = locale).to_string())
        }
        keys::KOSENJO_QUALIFYING_END => Some(t!(keys::KOSENJO_QUALIFYING_END, locale = locale).to_string()),
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
        keys::KOSENJO_SP_BATTLE_END => Some(t!(keys::KOSENJO_SP_BATTLE_END, locale = locale).to_string()),
        keys::KOSENJO_TEAM_ABILITY_1 => Some(t!(keys::KOSENJO_TEAM_ABILITY_1, locale = locale).to_string()),
        keys::KOSENJO_TEAM_ABILITY_2 => Some(t!(keys::KOSENJO_TEAM_ABILITY_2, locale = locale).to_string()),

        // Dorebara messages
        keys::DOREBARA_START => Some(t!(keys::DOREBARA_START, locale = locale).to_string()),
        keys::DOREBARA_END => Some(t!(keys::DOREBARA_END, locale = locale).to_string()),
        keys::DOREBARA_RESET => Some(t!(keys::DOREBARA_RESET, locale = locale).to_string()),
        keys::DOREBARA_VARIANT => Some(t!(keys::DOREBARA_VARIANT, locale = locale).to_string()),
        keys::DOREBARA_LAST_DAY => Some(t!(keys::DOREBARA_LAST_DAY, locale = locale).to_string()),

        // Bot messages
        keys::BOT_MENTION => Some(t!(keys::BOT_MENTION, locale = locale).to_string()),
        keys::BOT_MENTION_SIX => Some(t!(keys::BOT_MENTION_SIX, locale = locale).to_string()),
        keys::BOT_MENTION_CALLING => Some(t!(keys::BOT_MENTION_CALLING, locale = locale).to_string()),

        // Omikuji messages
        keys::OMIKUJI_HIHI => Some(t!(keys::OMIKUJI_HIHI, locale = locale).to_string()),
        keys::OMIKUJI_HAKYOKU => Some(t!(keys::OMIKUJI_HAKYOKU, locale = locale).to_string()),
        keys::OMIKUJI_OMEGA_UNIT => Some(t!(keys::OMIKUJI_OMEGA_UNIT, locale = locale).to_string()),

        // Recruitment display messages
        keys::RECRUITMENT_DISPLAY_NORMAL => Some(t!(keys::RECRUITMENT_DISPLAY_NORMAL, locale = locale).to_string()),
        keys::RECRUITMENT_DISPLAY_SIX_ELEMENTS => {
            Some(t!(keys::RECRUITMENT_DISPLAY_SIX_ELEMENTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_EVENT_DATE_LABEL => {
            Some(t!(keys::RECRUITMENT_DISPLAY_EVENT_DATE_LABEL, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_DATE_FORMAT => {
            Some(t!(keys::RECRUITMENT_DISPLAY_DATE_FORMAT, locale = locale).to_string())
        }
        keys::RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL => {
            Some(t!(keys::RECRUITMENT_DISPLAY_DISMISSAL_TIMES_LABEL, locale = locale).to_string())
        }
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
        keys::RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_BEFORE_5_MINUTES, locale = locale).to_string())
        }
        keys::RECRUITMENT_NOTIFICATION_START => Some(t!(keys::RECRUITMENT_NOTIFICATION_START, locale = locale).to_string()),
        keys::RECRUITMENT_NOTIFICATION_DISMISSAL => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_DISMISSAL, locale = locale).to_string())
        }
        keys::RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS => {
            Some(t!(keys::RECRUITMENT_NOTIFICATION_DISMISSAL_WITH_PARTICIPANTS, locale = locale).to_string())
        }

        // Timezone messages
        keys::TIMEZONE_SET_SUCCESS => Some(t!(keys::TIMEZONE_SET_SUCCESS, locale = locale).to_string()),
        keys::TIMEZONE_SHOW_CURRENT => Some(t!(keys::TIMEZONE_SHOW_CURRENT, locale = locale).to_string()),

        // Guild settings messages
        keys::GUILD_SETTINGS_SET_SUCCESS => Some(t!(keys::GUILD_SETTINGS_SET_SUCCESS, locale = locale).to_string()),
        keys::GUILD_SETTINGS_SHOW_SUCCESS => Some(t!(keys::GUILD_SETTINGS_SHOW_SUCCESS, locale = locale).to_string()),
        keys::GUILD_SETTINGS_NOT_SET => Some(t!(keys::GUILD_SETTINGS_NOT_SET, locale = locale).to_string()),

        // Recruitment role messages
        keys::RECRUITMENT_ROLE_ADD_SUCCESS => {
            Some(t!(keys::RECRUITMENT_ROLE_ADD_SUCCESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_ROLE_REMOVE_SUCCESS => {
            Some(t!(keys::RECRUITMENT_ROLE_REMOVE_SUCCESS, locale = locale).to_string())
        }

        // Recruitment command messages
        keys::RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_ALREADY_CANCELLED, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_MESSAGE_DELETED, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_INVALID_MESSAGE, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_NOT_FOUND => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_NOT_FOUND, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_ERROR => Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_ERROR, locale = locale).to_string()),
        keys::RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCELLED_MESSAGE_SUFFIX, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_NO_PARTICIPANTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCEL_NOTIFICATION_WITH_PARTICIPANTS, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CANCELLING_PROGRESS => {
            Some(t!(keys::RECRUITMENT_COMMAND_CANCELLING_PROGRESS, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CHANGE_NO_CHANGES => {
            Some(t!(keys::RECRUITMENT_COMMAND_CHANGE_NO_CHANGES, locale = locale).to_string())
        }
        keys::RECRUITMENT_COMMAND_CHANGE_SUCCESS => Some(t!(keys::RECRUITMENT_COMMAND_CHANGE_SUCCESS, locale = locale).to_string()),

        // General messages
        keys::MESSAGES_WELCOME => Some(t!(keys::MESSAGES_WELCOME, locale = locale).to_string()),
        keys::MESSAGES_HELP => Some(t!(keys::MESSAGES_HELP, locale = locale).to_string()),

        // 未知のメッセージID
        _ => None,
    }
}
