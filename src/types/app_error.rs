/// アプリケーション全体のエラー型（thiserror使用）
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Discord API error: {0}")]
    Discord(Box<poise::serenity_prelude::Error>),

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
    DiscordOperation(Box<crate::types::DiscordOperationError>),

    #[error("Channel creation failed")]
    ChannelCreationFailed,

    #[error("Command executed in category channel")]
    InCategoryChannelError,
}

impl From<poise::serenity_prelude::Error> for AppError {
    fn from(err: poise::serenity_prelude::Error) -> Self {
        AppError::Discord(Box::new(err))
    }
}

impl From<crate::types::DiscordOperationError> for AppError {
    fn from(err: crate::types::DiscordOperationError) -> Self {
        AppError::DiscordOperation(Box::new(err))
    }
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

impl From<crate::errors::ServiceError> for AppError {
    fn from(err: crate::errors::ServiceError) -> Self {
        match err {
            crate::errors::ServiceError::NotFound(msg) => AppError::NotFound(msg),
            crate::errors::ServiceError::Validation(e) => AppError::Business {
                message: e.to_string(),
            },
            crate::errors::ServiceError::BusinessRule(e) => AppError::Business {
                message: e.to_string(),
            },
            crate::errors::ServiceError::Database(msg) => AppError::Generic(msg),
            crate::errors::ServiceError::Internal(msg) => AppError::Generic(msg),
            crate::errors::ServiceError::ExternalService(e) => AppError::Generic(e.to_string()),
        }
    }
}

impl AppError {
    /// Discord上でユーザーに表示するメッセージを取得
    /// 技術的な詳細は含めず、ユーザーフレンドリーなメッセージのみ返す
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(_) => {
                "データベースエラーが発生しました。管理者に連絡してください。".to_string()
            }
            AppError::Discord(_) => {
                "Discord APIエラーが発生しました。しばらく待ってから再度お試しください。"
                    .to_string()
            }
            AppError::Business { message } => message.clone(),
            AppError::Config { message } => {
                format!("設定エラー: {message}")
            }
            AppError::Validation { field } => {
                format!("入力エラー: {field} の値が不正です。")
            }
            AppError::Generic(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::DiscordOperation(e) => {
                format!("Discord操作エラー: {e}")
            }
            AppError::ChannelCreationFailed => "チャンネルの作成に失敗しました。".to_string(),
            AppError::InCategoryChannelError => {
                "このコマンドはカテゴリ外のチャンネルで実行してください。".to_string()
            }
        }
    }
}

// TODO: Result -> AppResultへリネーム
pub type Result<T> = std::result::Result<T, AppError>;
