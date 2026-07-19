//! 自動募集カテゴリ設定Facade
//!
//! カテゴリ登録/解除/日数変更の処理を行う

mod change_days;
mod common;
mod messages;
mod register;
mod unregister;

pub use change_days::change_days;
pub use register::{CategoryRegistrationResult, register_category};
pub use unregister::unregister_category;
