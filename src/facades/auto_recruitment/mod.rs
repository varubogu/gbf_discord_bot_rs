//! 自動募集機能のFacade層
//!
//! コマンドやコンポーネントハンドラからの呼び出しを受け、
//! トランザクション管理とサービス層の協調を行う

pub mod category_setup_facade;
pub mod interaction_facade;
pub mod matching_check_facade;
pub mod quest_selection_facade;
pub mod status_facade;
pub mod time_selection_facade;

pub use category_setup_facade::{change_days, register_category, unregister_category};
pub use interaction_facade::{
    get_selected_quests, get_time_channel_date, register_selected_elements, toggle_quest_join,
};
pub use matching_check_facade::{
    check_and_notify_after_quest_selection, check_and_notify_after_time_selection,
};
pub use quest_selection_facade::handle_quest_selection;
pub use status_facade::get_participation_status;
pub use time_selection_facade::handle_time_selection;
