//! リポジトリコンテナ
//!
//! データベースリポジトリ群を保持するコンテナ。
//! Gateway抽象化には依存せず、DatabaseConnectionのみに依存する。

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::repository::database::{
    all_recruitment_notification_roles_repository::SeaOrmAllRecruitmentNotificationRolesRepository,
    auto_recruitment::{
        SeaOrmAutoRecruitmentChannelRepository, SeaOrmAutoRecruitmentParticipantRepository,
        SeaOrmAutoRecruitmentQuestMessageRepository, SeaOrmAutoRecruitmentRepository,
        SeaOrmQuestMatchingRepository, SeaOrmQuestMatchingUserRepository,
        SeaOrmUserDesiredQuestRepository,
    },
    battle_recruitments_repository::SeaOrmBattleRecruitmentsRepository,
    battle_style_repository::SeaOrmBattleStyleRepository,
    channel_type_repository::SeaOrmChannelTypeRepository,
    guild_channel_repository::SeaOrmGuildChannelRepository,
    guild_environment_repository::SeaOrmGuildEnvironmentRepository,
    guild_message_text_repository::SeaOrmGuildMessageTextRepository,
    guild_quest_disable_repository::SeaOrmGuildQuestDisableRepository,
    guild_repository::SeaOrmGuildRepository,
    guild_settings_repository::SeaOrmGuildSettingsRepository,
    last_process_time_repository::SeaOrmLastProcessTimeRepository,
    message_text_repository::SeaOrmMessageTextRepository,
    quest_recruitment_notification_roles_repository::SeaOrmQuestRecruitmentNotificationRolesRepository,
    quest_repository::SeaOrmQuestRepository,
    recruitment_participants_repository::SeaOrmRecruitmentParticipantsRepository,
    schedule::{
        SeaOrmBattleRecruitmentDismissalRepository,
        SeaOrmBattleRecruitmentScheduleDismissalRepository,
        SeaOrmBattleRecruitmentScheduleRepository,
        SeaOrmNotificationRelBattleRecruitmentRepository,
        SeaOrmNotificationRelEventScheduleRepository, SeaOrmNotificationRepository,
        SeaOrmScheduleRepository, SeaOrmScheduledTaskCleanupRepository,
        SeaOrmScheduledTaskDismissalRepository, SeaOrmScheduledTaskDissolutionRepository,
        SeaOrmScheduledTaskRecurringRecruitmentRepository, SeaOrmScheduledTaskRepository,
    },
};

/// リポジトリ群を保持するコンテナ
///
/// データベースアクセスを一元管理する。
/// Gateway抽象化には依存しない。
/// 各リポジトリはステートレスなunit structのため、Arcは不要でCopyで渡せる。
#[derive(Debug, Clone, Copy)]
pub struct Repositories {
    // === 基本リポジトリ ===
    pub battle_recruitments: SeaOrmBattleRecruitmentsRepository,
    pub recruitment_participants: SeaOrmRecruitmentParticipantsRepository,
    pub guild: SeaOrmGuildRepository,
    pub guild_settings: SeaOrmGuildSettingsRepository,
    pub guild_channel: SeaOrmGuildChannelRepository,
    pub guild_environment: SeaOrmGuildEnvironmentRepository,
    pub guild_message_text: SeaOrmGuildMessageTextRepository,
    pub guild_quest_disable: SeaOrmGuildQuestDisableRepository,
    pub quest: SeaOrmQuestRepository,
    pub battle_style: SeaOrmBattleStyleRepository,
    pub channel_type: SeaOrmChannelTypeRepository,
    pub message_text: SeaOrmMessageTextRepository,
    pub last_process_time: SeaOrmLastProcessTimeRepository,

    // === 通知ロール関連リポジトリ ===
    pub all_recruitment_notification_roles: SeaOrmAllRecruitmentNotificationRolesRepository,
    pub quest_recruitment_notification_roles: SeaOrmQuestRecruitmentNotificationRolesRepository,

    // === スケジュール関連リポジトリ ===
    pub scheduled_task: SeaOrmScheduledTaskRepository,
    pub scheduled_task_dismissal: SeaOrmScheduledTaskDismissalRepository,
    pub scheduled_task_dissolution: SeaOrmScheduledTaskDissolutionRepository,
    pub scheduled_task_cleanup: SeaOrmScheduledTaskCleanupRepository,
    pub scheduled_task_recurring: SeaOrmScheduledTaskRecurringRecruitmentRepository,
    pub schedule: SeaOrmScheduleRepository,
    pub battle_recruitment_schedule: SeaOrmBattleRecruitmentScheduleRepository,
    pub battle_recruitment_dismissal: SeaOrmBattleRecruitmentDismissalRepository,
    pub battle_recruitment_schedule_dismissal: SeaOrmBattleRecruitmentScheduleDismissalRepository,
    pub notification: SeaOrmNotificationRepository,
    pub notification_rel_battle_recruitment: SeaOrmNotificationRelBattleRecruitmentRepository,
    pub notification_rel_event_schedule: SeaOrmNotificationRelEventScheduleRepository,

    // === 自動募集関連リポジトリ ===
    pub auto_recruitment: SeaOrmAutoRecruitmentRepository,
    pub auto_recruitment_channel: SeaOrmAutoRecruitmentChannelRepository,
    pub auto_recruitment_participant: SeaOrmAutoRecruitmentParticipantRepository,
    pub auto_recruitment_quest_message: SeaOrmAutoRecruitmentQuestMessageRepository,
    pub quest_matching: SeaOrmQuestMatchingRepository,
    pub quest_matching_user: SeaOrmQuestMatchingUserRepository,
    pub user_desired_quest: SeaOrmUserDesiredQuestRepository,
}

impl Repositories {
    /// 新しいRepositoriesを作成する
    pub fn new(
        _guild_db: Arc<DatabaseConnection>,
        _system_db: Arc<DatabaseConnection>,
        _global_db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            // 基本リポジトリ
            battle_recruitments: SeaOrmBattleRecruitmentsRepository::new(),
            recruitment_participants: SeaOrmRecruitmentParticipantsRepository::new(),
            guild: SeaOrmGuildRepository::new(),
            guild_settings: SeaOrmGuildSettingsRepository::new(),
            guild_channel: SeaOrmGuildChannelRepository::new(),
            guild_environment: SeaOrmGuildEnvironmentRepository::new(),
            guild_message_text: SeaOrmGuildMessageTextRepository::new(),
            guild_quest_disable: SeaOrmGuildQuestDisableRepository::new(),
            quest: SeaOrmQuestRepository::new(),
            battle_style: SeaOrmBattleStyleRepository::new(),
            channel_type: SeaOrmChannelTypeRepository::new(),
            message_text: SeaOrmMessageTextRepository::new(),
            last_process_time: SeaOrmLastProcessTimeRepository::new(),

            // 通知ロール関連リポジトリ
            all_recruitment_notification_roles:
                SeaOrmAllRecruitmentNotificationRolesRepository::new(),
            quest_recruitment_notification_roles:
                SeaOrmQuestRecruitmentNotificationRolesRepository::new(),

            // スケジュール関連リポジトリ
            scheduled_task: SeaOrmScheduledTaskRepository::new(),
            scheduled_task_dismissal: SeaOrmScheduledTaskDismissalRepository::new(),
            scheduled_task_dissolution: SeaOrmScheduledTaskDissolutionRepository::new(),
            scheduled_task_cleanup: SeaOrmScheduledTaskCleanupRepository::new(),
            scheduled_task_recurring: SeaOrmScheduledTaskRecurringRecruitmentRepository::new(),
            schedule: SeaOrmScheduleRepository::new(),
            battle_recruitment_schedule: SeaOrmBattleRecruitmentScheduleRepository::new(),
            battle_recruitment_dismissal: SeaOrmBattleRecruitmentDismissalRepository::new(),
            battle_recruitment_schedule_dismissal:
                SeaOrmBattleRecruitmentScheduleDismissalRepository::new(),
            notification: SeaOrmNotificationRepository::new(),
            notification_rel_battle_recruitment:
                SeaOrmNotificationRelBattleRecruitmentRepository::new(),
            notification_rel_event_schedule: SeaOrmNotificationRelEventScheduleRepository::new(),

            // 自動募集関連リポジトリ
            auto_recruitment: SeaOrmAutoRecruitmentRepository::new(),
            auto_recruitment_channel: SeaOrmAutoRecruitmentChannelRepository::new(),
            auto_recruitment_participant: SeaOrmAutoRecruitmentParticipantRepository::new(),
            auto_recruitment_quest_message: SeaOrmAutoRecruitmentQuestMessageRepository::new(),
            quest_matching: SeaOrmQuestMatchingRepository::new(),
            quest_matching_user: SeaOrmQuestMatchingUserRepository::new(),
            user_desired_quest: SeaOrmUserDesiredQuestRepository::new(),
        }
    }
}
