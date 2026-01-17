use crate::types::{AppError, DbRole};

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub discord_token: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    /// イベント期間外のスケジュール作成を許可する最大日数（デフォルト: 365日）
    pub max_schedule_days_outside_event: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let discord_token = std::env::var("DISCORD_TOKEN").map_err(|_| AppError::Config {
            message: "DISCORD_TOKEN not set".to_string(),
        })?;

        let db_host = std::env::var("DB_HOST").map_err(|_| AppError::Config {
            message: "DB_HOST not set".to_string(),
        })?;

        let db_port = std::env::var("DB_PORT")
            .map_err(|_| AppError::Config {
                message: "DB_PORT not set".to_string(),
            })?
            .parse::<u16>()
            .map_err(|_| AppError::Config {
                message: "DB_PORT must be a valid port number".to_string(),
            })?;

        let db_name = std::env::var("DB_NAME").map_err(|_| AppError::Config {
            message: "DB_NAME not set".to_string(),
        })?;

        // イベント期間外のスケジュール作成を許可する最大日数
        // 環境変数がない場合は365日をデフォルトとする
        let max_schedule_days_outside_event = std::env::var("MAX_SCHEDULE_DAYS_OUTSIDE_EVENT")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(365);

        // 最大日数の妥当性チェック（365日を超える場合はエラー）
        if max_schedule_days_outside_event > 365 {
            return Err(AppError::Config {
                message: format!(
                    "MAX_SCHEDULE_DAYS_OUTSIDE_EVENT must be 365 or less, but got: {}",
                    max_schedule_days_outside_event
                ),
            });
        }

        // 負の値もエラーとする
        if max_schedule_days_outside_event < 0 {
            return Err(AppError::Config {
                message: format!(
                    "MAX_SCHEDULE_DAYS_OUTSIDE_EVENT must be non-negative, but got: {}",
                    max_schedule_days_outside_event
                ),
            });
        }

        Ok(Self {
            discord_token,
            db_host,
            db_port,
            db_name,
            max_schedule_days_outside_event,
        })
    }

    /// データベース接続URLを構築（指定されたロール用）
    ///
    /// # Arguments
    /// * `role` - 使用するデータベースロール
    ///
    /// # Examples
    /// ```no_run
    /// use gbf_discord_bot_rs::types::{AppConfig, DbRole};
    ///
    /// let config = AppConfig::from_env()?;
    /// // 通常のコマンド実行用
    /// let url = config.database_url(DbRole::Guild)?;
    /// // スケジューラー用
    /// let url = config.database_url(DbRole::System)?;
    /// # Ok::<(), gbf_discord_bot_rs::types::AppError>(())
    /// ```
    pub fn database_url(&self, role: DbRole) -> Result<String, AppError> {
        let username = role.username()?;
        let password = role.password()?;

        Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            username, password, self.db_host, self.db_port, self.db_name
        ))
    }
}
