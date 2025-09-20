/// アプリケーション全体のエラー型（thiserror使用）
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Discord API error: {0}")]
    Discord(#[from] poise::serenity_prelude::Error),

    #[error("Business logic error: {message}")]
    Business { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Validation error: {field} is invalid")]
    Validation { field: String },

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Discord operation error: {0}")]
    DiscordOperation(#[from] crate::types::DiscordOperationError),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Generic(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Generic(err)
    }
}

// TODO: Result -> AppResultへリネーム
pub type Result<T> = std::result::Result<T, AppError>;
