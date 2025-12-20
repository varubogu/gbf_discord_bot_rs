use crate::types::AppError;
use std::env;

/// データベース接続ロール
///
/// 用途に応じて適切なロールを選択してデータベース接続を行う
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbRole {
    /// システム処理用（スケジューラー等）
    /// - master: SELECT only（読み取り専用）
    /// - guild_master/worker: 全操作可能
    /// - RLS適用なし（BYPASSRLS）
    System,

    /// 通常のコマンド実行用
    /// - master: SELECT only（読み取り専用）
    /// - guild_master/worker: 全操作可能
    /// - RLS適用あり（guild_id制限）
    Guild,

    /// グローバルデータ更新用（スプレッドシート同期等）
    /// - master: 全操作可能（マスターデータ更新）
    /// - guild_master/worker: 全操作可能
    /// - RLS適用なし（BYPASSRLS）
    Global,

    /// マイグレーション・管理用
    /// - 全スキーマ: 全操作可能
    /// - RLS適用なし（BYPASSRLS）
    Admin,
}

impl DbRole {
    /// 環境変数からユーザー名を取得
    pub fn username(&self) -> Result<String, AppError> {
        let var_name = match self {
            DbRole::System => "SYSTEM_DB_USER",
            DbRole::Guild => "GUILD_DB_USER",
            DbRole::Global => "GLOBAL_DB_USER",
            DbRole::Admin => "ADMIN_DB_USER",
        };

        env::var(var_name).map_err(|_| AppError::Config {
            message: format!("{var_name} not set"),
        })
    }

    /// 環境変数からパスワードを取得
    pub fn password(&self) -> Result<String, AppError> {
        let var_name = match self {
            DbRole::System => "SYSTEM_DB_PASSWORD",
            DbRole::Guild => "GUILD_DB_PASSWORD",
            DbRole::Global => "GLOBAL_DB_PASSWORD",
            DbRole::Admin => "ADMIN_DB_PASSWORD",
        };

        env::var(var_name).map_err(|_| AppError::Config {
            message: format!("{var_name} not set"),
        })
    }

    /// ロールの説明を取得
    pub fn description(&self) -> &'static str {
        match self {
            DbRole::System => "システム処理用（スケジューラー等）",
            DbRole::Guild => "通常のコマンド実行用（RLS適用）",
            DbRole::Global => "グローバルデータ更新用（スプレッドシート同期等）",
            DbRole::Admin => "マイグレーション・管理用",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_role_username_env_var() {
        assert_eq!(DbRole::System.username().is_err(), true);

        unsafe {
            env::set_var("SYSTEM_DB_USER", "gbf_bot_system");
        }
        assert_eq!(DbRole::System.username().unwrap(), "gbf_bot_system");

        unsafe {
            env::remove_var("SYSTEM_DB_USER");
        }
    }

    #[test]
    fn test_db_role_password_env_var() {
        assert_eq!(DbRole::Guild.password().is_err(), true);

        unsafe {
            env::set_var("GUILD_DB_PASSWORD", "test_password");
        }
        assert_eq!(DbRole::Guild.password().unwrap(), "test_password");

        unsafe {
            env::remove_var("GUILD_DB_PASSWORD");
        }
    }

    #[test]
    fn test_db_role_description() {
        assert!(DbRole::System.description().contains("システム処理用"));
        assert!(DbRole::Guild.description().contains("通常のコマンド実行用"));
        assert!(
            DbRole::Global
                .description()
                .contains("グローバルデータ更新用")
        );
        assert!(DbRole::Admin.description().contains("マイグレーション"));
    }
}
