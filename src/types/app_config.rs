use crate::types::AppError;

/// アプリケーション設定
#[derive(Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub discord_token: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| AppError::Config {
            message: "DATABASE_URL not set".to_string(),
        })?;

        let discord_token = std::env::var("DISCORD_TOKEN").map_err(|_| AppError::Config {
            message: "DISCORD_TOKEN not set".to_string(),
        })?;

        Ok(Self {
            database_url,
            discord_token,
        })
    }
}
