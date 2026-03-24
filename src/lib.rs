pub mod constants;
pub mod di;
pub mod errors;
pub mod events;
pub mod facades;
pub mod gateway;
pub mod infrastructure;
pub mod models;
pub mod presenter;
pub mod repository;
pub mod services;
pub mod types;
pub mod utils;

// Library interface for gbf_discord_bot_rs
// This exposes modules for use by examples and external code

// Initialize rust-i18n at the library root
rust_i18n::i18n!("locales");

// Test utilities for integration tests
pub mod test_utils {
    use crate::services::unified_datetime_parser::{
        DateTimeParseOptions, ParsedDateTime, parse_datetime,
    };
    use crate::types::DbRole;
    use chrono::{DateTime, Local, Timelike};
    use chrono_tz::Tz;
    use sea_orm::{Database, DatabaseConnection, DbErr};
    use std::env;

    /// Test utility to check database availability
    pub fn check_database_availability() -> (bool, Vec<String>) {
        check_database_availability_for_roles(&[
            DbRole::System,
            DbRole::Guild,
            DbRole::Global,
            DbRole::Admin,
        ])
    }

    /// テストで必要なロール別DB環境変数の利用可否を確認する
    pub fn check_database_availability_for_roles(roles: &[DbRole]) -> (bool, Vec<String>) {
        let mut required_vars = vec!["DB_HOST", "DB_PORT", "DB_NAME"];

        for role in roles {
            let (user_var, password_var) = role_credential_env_names(*role);
            required_vars.push(user_var);
            required_vars.push(password_var);
        }

        required_vars.sort_unstable();
        required_vars.dedup();

        let missing = required_vars
            .into_iter()
            .filter(|var_name| env::var(var_name).is_err())
            .map(str::to_string)
            .collect::<Vec<_>>();

        (missing.is_empty(), missing)
    }

    /// テスト用に指定ロールのDBへ接続する
    pub async fn connect_database_for_role(role: DbRole) -> Result<DatabaseConnection, DbErr> {
        let database_url = build_database_url_for_role(role).map_err(DbErr::Custom)?;
        Database::connect(&database_url).await
    }

    /// ロール別認証情報の環境変数名を取得する
    fn role_credential_env_names(role: DbRole) -> (&'static str, &'static str) {
        match role {
            DbRole::System => ("SYSTEM_DB_USER", "SYSTEM_DB_PASSWORD"),
            DbRole::Guild => ("GUILD_DB_USER", "GUILD_DB_PASSWORD"),
            DbRole::Global => ("GLOBAL_DB_USER", "GLOBAL_DB_PASSWORD"),
            DbRole::Admin => ("ADMIN_DB_USER", "ADMIN_DB_PASSWORD"),
        }
    }

    /// テスト用に指定ロールのDB接続URLを構築する
    fn build_database_url_for_role(role: DbRole) -> Result<String, String> {
        let (available, missing) = check_database_availability_for_roles(&[role]);
        if !available {
            return Err(format!(
                "テスト用DB接続情報が不足しています: {}",
                missing.join(", ")
            ));
        }

        let host = env::var("DB_HOST").map_err(|_| "DB_HOST が設定されていません".to_string())?;
        let port = env::var("DB_PORT").map_err(|_| "DB_PORT が設定されていません".to_string())?;
        let database =
            env::var("DB_NAME").map_err(|_| "DB_NAME が設定されていません".to_string())?;

        let (user_var, password_var) = role_credential_env_names(role);
        let user = env::var(user_var).map_err(|_| format!("{user_var} が設定されていません"))?;
        let password =
            env::var(password_var).map_err(|_| format!("{password_var} が設定されていません"))?;

        Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, host, port, database
        ))
    }

    /// Test utility to get default expiry date (今日21:00)
    pub fn get_default_expiry_date() -> DateTime<Local> {
        let now = Local::now();
        now.with_hour(21)
            .unwrap_or(now)
            .with_minute(0)
            .unwrap_or(now)
            .with_second(0)
            .unwrap_or(now)
            .with_nanosecond(0)
            .unwrap_or(now)
    }

    /// YAMLメッセージを取得するテスト用ユーティリティ
    ///
    /// `yaml_loader::get_yaml_message` を統合テストから呼び出すためのラッパー。
    /// 統合テストからクレート内部モジュールにアクセスするためにここで公開する。
    pub fn resolve_yaml_message(message_id: &str, locale: &str) -> Option<String> {
        crate::services::message::yaml_loader::get_yaml_message(message_id, locale)
    }

    /// Test utility to parse event date using unified_datetime_parser
    pub fn parse_event_date(date_str: &str) -> Result<DateTime<Local>, String> {
        let trimmed_input = date_str.trim();
        if trimmed_input.is_empty() {
            return Ok(get_default_expiry_date());
        }

        // デフォルトタイムゾーンとしてAsia/Tokyoを使用
        let timezone: Tz = "Asia/Tokyo"
            .parse()
            .map_err(|e| format!("タイムゾーン解析に失敗しました: {e}"))?;
        let options = DateTimeParseOptions::for_quest_departure(timezone);

        let results = parse_datetime(trimmed_input, &options)
            .map_err(|e| format!("Failed to parse date string '{trimmed_input}': {e}"))?;

        if results.is_empty() {
            return Err(format!("No valid date parsed from '{trimmed_input}'"));
        }

        // 最初の結果を使用
        match &results[0] {
            ParsedDateTime::Absolute(dt) => {
                // unified_datetime_parserはAsia/Tokyoでパースしたものを
                // UTCに変換して返すため、Asia/Tokyoに戻してからLocalに変換
                let tokyo_tz: Tz = "Asia/Tokyo"
                    .parse()
                    .map_err(|e| format!("タイムゾーン解析に失敗しました: {e}"))?;
                let tokyo_dt = dt.with_timezone(&tokyo_tz);
                Ok(tokyo_dt.with_timezone(&Local))
            }
            ParsedDateTime::Time(time) => {
                // 時刻のみの場合、今日または明日の日付を設定
                let now = Local::now();
                let today_at_time = now
                    .date_naive()
                    .and_time(*time)
                    .and_local_timezone(Local)
                    .single()
                    .unwrap_or(now);

                if today_at_time > now {
                    Ok(today_at_time)
                } else {
                    // 過去の時刻なら明日
                    let tomorrow = now + chrono::Duration::days(1);
                    Ok(tomorrow
                        .date_naive()
                        .and_time(*time)
                        .and_local_timezone(Local)
                        .single()
                        .unwrap_or(now))
                }
            }
            ParsedDateTime::Relative { .. } => {
                Err("Relative time not supported in test utility".to_string())
            }
        }
    }
}
