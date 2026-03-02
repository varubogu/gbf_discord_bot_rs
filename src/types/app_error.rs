/// アプリケーション全体のエラー型（thiserror使用）
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("データベースエラー: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Discord APIエラー: {0}")]
    Discord(Box<poise::serenity_prelude::Error>),

    #[error("業務エラー: {message}")]
    Business { message: String },

    #[error("設定エラー: {message}")]
    Config { message: String },

    #[error("入力検証エラー: {field}")]
    Validation { field: String },

    #[error("汎用エラー: {0}")]
    Generic(String),

    #[error("未検出エラー: {0}")]
    NotFound(String),

    #[error("Discord操作エラー: {0}")]
    DiscordOperation(Box<crate::types::DiscordOperationError>),

    #[error("チャンネルの作成に失敗しました")]
    ChannelCreationFailed,

    #[error("このコマンドはカテゴリチャンネル内で実行できません")]
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

impl From<crate::errors::GatewayError> for AppError {
    fn from(err: crate::errors::GatewayError) -> Self {
        AppError::Generic(format!("ゲートウェイエラー: {err}"))
    }
}

impl From<crate::errors::RecruitmentError> for AppError {
    fn from(err: crate::errors::RecruitmentError) -> Self {
        AppError::Business {
            message: err.to_string(),
        }
    }
}

impl From<crate::errors::ScheduleError> for AppError {
    fn from(err: crate::errors::ScheduleError) -> Self {
        AppError::Business {
            message: err.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{RecruitmentError, ScheduleError};

    #[test]
    fn recruitment_errorからapp_errorへ変換できる() {
        let err = AppError::from(RecruitmentError::InvalidCustomId);
        match err {
            AppError::Business { message } => {
                assert!(message.contains("不正なカスタムID"));
            }
            other => panic!("想定外のエラー型です: {other}"),
        }
    }

    #[test]
    fn schedule_errorからapp_errorへ変換できる() {
        let err = AppError::from(ScheduleError::DispatchFailed);
        match err {
            AppError::Business { message } => {
                assert!(message.contains("スケジュールディスパッチ"));
            }
            other => panic!("想定外のエラー型です: {other}"),
        }
    }

    #[test]
    fn app_error表示文言は日本語である() {
        let err = AppError::Database(sea_orm::DbErr::Custom("接続失敗".to_string()));
        assert!(err.to_string().contains("データベースエラー"));
    }
}
