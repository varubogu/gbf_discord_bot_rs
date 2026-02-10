/// Service層エラー
///
/// ビジネスロジック関連のエラーを定義します。
use thiserror::Error;

/// バリデーションエラー
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("必須フィールドが未入力です: {field}")]
    RequiredFieldMissing { field: String },

    #[error("フィールドの値が範囲外です: {field} (値: {value}, 許容範囲: {range})")]
    ValueOutOfRange {
        field: String,
        value: String,
        range: String,
    },

    #[error("フィールドの形式が正しくありません: {field} (理由: {reason})")]
    InvalidFormat { field: String, reason: String },

    #[error("データ型変換エラー: {field} (値: {value}, 期待される型: {expected_type})")]
    TypeConversionError {
        field: String,
        value: String,
        expected_type: String,
    },

    #[error("日時形式エラー: {value} (対応フォーマット: {supported_formats})")]
    DateTimeFormatError {
        value: String,
        supported_formats: String,
    },

    #[error("UUID形式エラー: {value}")]
    UuidFormatError { value: String },

    #[error("外部キー制約エラー: {field} (参照先: {reference_table}, 値: {value})")]
    ForeignKeyViolation {
        field: String,
        reference_table: String,
        value: String,
    },
}

/// ビジネスルール違反エラー
#[derive(Error, Debug)]
pub enum BusinessRuleError {
    #[error("権限がありません: {required_permission}")]
    InsufficientPermission { required_permission: String },

    #[error("募集が既に満員です (募集ID: {recruitment_id}, 定員: {capacity})")]
    RecruitmentFull {
        recruitment_id: String,
        capacity: i32,
    },

    #[error("重複した操作です: {operation}")]
    DuplicateOperation { operation: String },

    #[error("操作対象が不正な状態です: {entity} (現在の状態: {current_state})")]
    InvalidState {
        entity: String,
        current_state: String,
    },

    #[error("ギルドIDが一致しません (期待: {expected}, 実際: {actual})")]
    GuildIdMismatch { expected: String, actual: String },

    #[error("テーブル定義エラー: {table_name} (理由: {reason})")]
    TableDefinitionError { table_name: String, reason: String },
}

/// 外部サービスエラー
#[derive(Error, Debug)]
pub enum ExternalServiceError {
    #[error("Discord APIエラー: {message}")]
    DiscordApiError { message: String },

    #[error("Google Sheets APIエラー: {message}")]
    GoogleSheetsApiError { message: String },

    #[error("Google認証エラー: {message}")]
    GoogleAuthError { message: String },

    #[error("スプレッドシートが見つかりません: {spreadsheet_url}")]
    SpreadsheetNotFound { spreadsheet_url: String },

    #[error("シートが見つかりません: {sheet_name} (スプレッドシート: {spreadsheet_id})")]
    SheetNotFound {
        sheet_name: String,
        spreadsheet_id: String,
    },

    #[error("外部サービスタイムアウト: {service_name} (タイムアウト: {timeout_seconds}秒)")]
    ServiceTimeout {
        service_name: String,
        timeout_seconds: u64,
    },

    #[error("ネットワークエラー: {message}")]
    NetworkError { message: String },
}

/// Service層統合エラー
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("バリデーションエラー: {0}")]
    Validation(#[from] ValidationError),

    #[error("ビジネスルールエラー: {0}")]
    BusinessRule(#[from] BusinessRuleError),

    #[error("外部サービスエラー: {0}")]
    ExternalService(#[from] ExternalServiceError),

    #[error("データが見つかりません: {0}")]
    NotFound(String),

    #[error("データベースエラー: {0}")]
    Database(String),

    #[error("内部エラー: {0}")]
    Internal(String),
}

// Repository層エラーからビジネスルールエラーへの変換
impl From<crate::errors::RepositoryError> for BusinessRuleError {
    fn from(err: crate::errors::RepositoryError) -> Self {
        match err {
            crate::errors::RepositoryError::NotFound { entity_type, id } => {
                BusinessRuleError::InvalidState {
                    entity: entity_type,
                    current_state: format!("ID {id} が見つかりません"),
                }
            }
            crate::errors::RepositoryError::ConstraintViolation { constraint } => {
                BusinessRuleError::DuplicateOperation {
                    operation: format!("制約違反: {constraint}"),
                }
            }
            _ => BusinessRuleError::InvalidState {
                entity: "Unknown".to_string(),
                current_state: "データアクセスエラー".to_string(),
            },
        }
    }
}

// AppErrorからServiceErrorへの変換
impl From<crate::types::AppError> for ServiceError {
    fn from(err: crate::types::AppError) -> Self {
        match err {
            crate::types::AppError::Database(e) => ServiceError::Database(e.to_string()),
            crate::types::AppError::NotFound(msg) => ServiceError::NotFound(msg),
            crate::types::AppError::Business { message } => {
                ServiceError::BusinessRule(BusinessRuleError::InvalidState {
                    entity: "Unknown".to_string(),
                    current_state: message,
                })
            }
            crate::types::AppError::Validation { field } => {
                ServiceError::Validation(ValidationError::InvalidFormat {
                    field,
                    reason: "Validation failed".to_string(),
                })
            }
            crate::types::AppError::Discord(e) => {
                ServiceError::ExternalService(ExternalServiceError::DiscordApiError {
                    message: e.to_string(),
                })
            }
            crate::types::AppError::DiscordOperation(e) => {
                ServiceError::ExternalService(ExternalServiceError::DiscordApiError {
                    message: e.to_string(),
                })
            }
            _ => ServiceError::Internal(err.to_string()),
        }
    }
}

// SeaORM DbErrからServiceErrorへの変換
impl From<sea_orm::DbErr> for ServiceError {
    fn from(err: sea_orm::DbErr) -> Self {
        ServiceError::Database(err.to_string())
    }
}
