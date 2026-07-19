use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentQuestMessageRepository,
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::CategorySetupService;
use crate::types::AppState;
use rust_i18n::t;

use crate::services::message::MessageTextId;

/// `AppState`のリポジトリ群から`CategorySetupService`を構築する
///
/// register/unregister/change_daysの3操作全てで同一の構築ロジックが必要なため共通化した。
pub(super) fn build_setup_service(
    app_state: &AppState,
) -> CategorySetupService<
    impl AutoRecruitmentRepository,
    impl AutoRecruitmentChannelRepository,
    impl QuestRepository,
    impl AutoRecruitmentQuestMessageRepository,
    impl QuestMatchingUserRepository,
    impl QuestMatchingRepository,
    impl ScheduledTaskRepository,
> {
    CategorySetupService::new(
        app_state.repositories.auto_recruitment,
        app_state.repositories.auto_recruitment_channel,
        app_state.repositories.quest,
        app_state.repositories.auto_recruitment_quest_message,
        app_state.repositories.quest_matching_user,
        app_state.repositories.quest_matching,
        app_state.repositories.scheduled_task,
    )
}

/// 自動募集関連メッセージを日本語で取得する
pub(super) fn localized_ja(message_id: MessageTextId) -> String {
    t!(message_id.as_str(), locale = "ja").to_string()
}
