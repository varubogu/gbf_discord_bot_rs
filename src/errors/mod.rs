pub mod facade;
pub mod presentation;
/// エラー型定義モジュール
///
/// 設計書: docs/develop/design/error_types.md
pub mod repository;
pub mod service;

// Re-exports
pub use facade::FacadeError;
pub use presentation::PresentationError;
pub use repository::RepositoryError;
pub use service::{BusinessRuleError, ExternalServiceError, ServiceError, ValidationError};
