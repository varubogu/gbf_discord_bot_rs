//! 自動募集機能のサービス層

pub mod matching_service;
pub mod notification_service;

pub use matching_service::{MatchCandidate, MatchGroup, PeriodicMatchingService};
pub use notification_service::AutoRecruitmentNotificationService;
