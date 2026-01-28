//! Discord関連のドメイン型定義
//!
//! poise/serenityの型に依存しない、ビジネスロジック層で使用する型を定義する。
//! これにより、facade/service/repository層からpoise/serenityへの直接依存を排除できる。

mod autocomplete;
mod channel;
mod guild;
mod ids;
mod interaction;
mod message;
mod reaction;

pub use autocomplete::*;
pub use channel::*;
pub use guild::*;
pub use ids::*;
pub use interaction::*;
pub use message::*;
pub use reaction::*;
