/// Facade層エラー
///
/// 複数のService層エラーを統合します。

use thiserror::Error;

use super::{
    repository::RepositoryError, service::{BusinessRuleError, ExternalServiceError, ValidationError}
};

#[derive(Error, Debug)]
pub enum FacadeError {
    #[error("バリデーションエラー")]
    Validation {
        #[from]
        source: ValidationError,
    },

    #[error("ビジネスルールエラー")]
    BusinessRule {
        #[from]
        source: BusinessRuleError,
    },

    #[error("外部サービスエラー")]
    ExternalService {
        #[from]
        source: ExternalServiceError,
    },

    #[error("データアクセスエラー")]
    Repository {
        #[from]
        source: RepositoryError,
    },

    #[error("トランザクションエラー: {message}")]
    TransactionError { message: String },

    #[error("初期化エラー: {message}")]
    Initialization { message: String },

    #[error("データベースエラー")]
    Database {
        #[from]
        source: sea_orm::DbErr,
    },
}
