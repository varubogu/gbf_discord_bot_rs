/// エラー型定義モジュール
///
/// 設計書: docs/develop/design/error_types.md

pub mod repository;
pub mod service;
pub mod facade;
pub mod presentation;

// Re-exports
pub use repository::RepositoryError;
pub use service::{ValidationError, BusinessRuleError, ExternalServiceError};
pub use facade::FacadeError;
pub use presentation::PresentationError;
