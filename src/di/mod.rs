//! 依存性注入（DI）コンテナモジュール
//!
//! Gateway抽象化を通じた適切な依存性注入パターンを提供する。
//! 静的ディスパッチ（ジェネリクス）を採用し、本番用とテスト用で異なる具象型を使用可能。
//!
//! ## アーキテクチャ
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     main.rs / Bot Setup                      │
//! │         (DI Container: Gateway, Service, Facadeの構築)       │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!            ┌──────────────────┼──────────────────┐
//!            ▼                  ▼                  ▼
//!     ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//!     │   Gateway   │    │   Service   │    │   Facade    │
//!     │ (Arc<Http>) │◄───│  (Arc<G>)   │◄───│ (Arc<Svc>)  │
//!     └─────────────┘    └─────────────┘    └─────────────┘
//! ```

mod container;
mod repositories;

pub use container::AppContainer;
pub use repositories::Repositories;

// 本番用型エイリアス
use crate::gateway::PoiseDiscordGateway;

/// 本番環境用のアプリケーションコンテナ
pub type ProductionAppContainer = AppContainer<PoiseDiscordGateway>;
