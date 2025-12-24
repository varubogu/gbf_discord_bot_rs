use crate::types::{AppError, DbRole};

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub discord_token: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
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

        Ok(Self {
            discord_token,
            db_host,
            db_port,
            db_name,
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
