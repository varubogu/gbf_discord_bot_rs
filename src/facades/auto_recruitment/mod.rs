//! 自動募集機能のFacade層
//!
//! コマンドやコンポーネントハンドラからの呼び出しを受け、
//! トランザクション管理とサービス層の協調を行う

pub mod category_setup_facade;
pub mod quest_selection_facade;
pub mod status_facade;
pub mod time_selection_facade;
pub mod voting_facade;

pub use category_setup_facade::{change_days, register_category, unregister_category};
pub use quest_selection_facade::handle_quest_selection;
pub use status_facade::get_participation_status;
pub use time_selection_facade::handle_time_selection;
pub use voting_facade::handle_vote;
