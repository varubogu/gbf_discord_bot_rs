//! Discord Gateway抽象化層
//!
//! poise/serenityとビジネスロジック層の間のインターフェースを定義する。
//! この層により、facade/service/repository層はpoise/serenityに直接依存せず、
//! Gatewayトレイトを通じてDiscord APIを操作できる。
//!
//! ## アーキテクチャ
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Events Layer                           │
//! │  (poise commands, event handlers - poise依存OK)             │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Gateway Layer (このモジュール)            │
//! │  DiscordGateway trait + PoiseDiscordGateway impl            │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Facade / Service / Repository                   │
//! │         (Discord依存なし - Gateway traitのみ使用)            │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod discord_channel_gateway;
mod discord_gateway;
mod discord_guild_gateway;
mod discord_interaction_gateway;
mod discord_message_gateway;
mod discord_reaction_gateway;

pub mod r#impl;

// トレイト再エクスポート
pub use discord_channel_gateway::DiscordChannelGateway;
pub use discord_gateway::DiscordGateway;
pub use discord_guild_gateway::DiscordGuildGateway;
pub use discord_interaction_gateway::DiscordInteractionGateway;
pub use discord_message_gateway::DiscordMessageGateway;
pub use discord_reaction_gateway::DiscordReactionGateway;

// 実装の再エクスポート
pub use r#impl::poise_discord_gateway::PoiseDiscordGateway;
