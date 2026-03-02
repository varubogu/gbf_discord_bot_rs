pub mod facade;
pub mod gateway;
pub mod presentation;
pub mod recruitment;
/// エラー型定義モジュール
///
/// 設計書: docs/develop/design/error_types.md
pub mod repository;
pub mod schedule;
pub mod service;

// Re-exports
pub use facade::FacadeError;
pub use gateway::GatewayError;
pub use presentation::PresentationError;
pub use recruitment::RecruitmentError;
pub use repository::RepositoryError;
pub use schedule::ScheduleError;
pub use service::{BusinessRuleError, ExternalServiceError, ServiceError, ValidationError};
