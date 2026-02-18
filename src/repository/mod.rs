pub mod all_recruitment_notification_roles_repository;
pub mod auto_recruitment;
pub mod battle_recruitments_repository;
pub mod battle_style_repository;
pub mod channel_type_repository;
pub mod guild_channel_repository;
pub mod guild_environment_repository;
pub mod guild_message_text_repository;
pub mod guild_quest_disable_repository;
pub mod guild_repository;
pub mod guild_settings_repository;
pub mod guild_spreadsheet_config_repository;
pub mod last_process_time_repository;
pub mod message_text_repository;
pub mod quest_aliases_repository;
pub mod quest_recruitment_notification_roles_repository;
pub mod quest_repository;
pub mod recruitment_participants_repository;
pub mod schedule;

// 抽象インターフェースをre-export
pub use all_recruitment_notification_roles_repository::AllRecruitmentNotificationRolesRepository;
pub use auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentParticipantRepository,
    AutoRecruitmentQuestMessageRepository, AutoRecruitmentRepository, CreateAutoRecruitmentParams,
    QuestMatchingRepository, QuestMatchingUserRepository, UserDesiredQuestRepository,
};
pub use battle_recruitments_repository::{
    BattleRecruitmentsRepository, CreateBattleRecruitmentParams,
};
pub use battle_style_repository::BattleStyleRepository;
pub use channel_type_repository::ChannelTypeRepository;
pub use guild_channel_repository::GuildChannelRepository;
pub use guild_environment_repository::GuildEnvironmentRepository;
pub use guild_message_text_repository::GuildMessageTextRepository;
pub use guild_quest_disable_repository::GuildQuestDisableRepository;
pub use guild_repository::GuildRepository;
pub use guild_settings_repository::GuildSettingsRepository;
pub use guild_spreadsheet_config_repository::GuildSpreadsheetConfigRepositoryTrait;
pub use last_process_time_repository::LastProcessTimeRepository;
pub use message_text_repository::MessageTextRepository;
pub use quest_recruitment_notification_roles_repository::QuestRecruitmentNotificationRolesRepository;
pub use quest_repository::QuestRepository;
pub use recruitment_participants_repository::RecruitmentParticipantsRepository;

// schedule配下のtraitをre-export
pub use schedule::{
    BattleRecruitmentDismissalRepository, BattleRecruitmentScheduleDismissalRepository,
    BattleRecruitmentScheduleRepository, CleanupWithTask, CreateScheduleParams, DismissalWithTask,
    DissolutionWithTask, NotificationRelBattleRecruitmentRepository,
    NotificationRelEventScheduleRepository, NotificationRepository, RecurringRecruitmentWithTask,
    ScheduleRepository, ScheduledTaskCleanupRepository, ScheduledTaskDismissalRepository,
    ScheduledTaskDissolutionRepository, ScheduledTaskRecurringRecruitmentRepository,
    ScheduledTaskRepository,
};
