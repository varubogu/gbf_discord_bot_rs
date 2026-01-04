pub mod all_recruitment_notification_roles_repository;
pub mod battle_recruitments_repository;
pub mod battle_style_repository;
pub mod channel_type_repository;
pub mod database;
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
pub use guild_spreadsheet_config_repository::{
    GuildSpreadsheetConfigRepository, GuildSpreadsheetConfigRepositoryTrait,
};
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
    NotificationRelEventScheduleRepository, NotificationRepository, NotificationWithTask,
    RecurringRecruitmentWithTask, ScheduleRepository, ScheduledTaskCleanupRepository,
    ScheduledTaskDismissalRepository, ScheduledTaskDissolutionRepository,
    ScheduledTaskNotificationRepository, ScheduledTaskRecurringRecruitmentRepository,
    ScheduledTaskRepository,
};

// /// リポジトリファクトリ
// /// データベース実装の詳細を隠蔽し、抽象インターフェースのみ公開
// pub struct RepositoryFactory;

// impl RepositoryFactory {
//     /// バトル募集リポジトリを作成
//     pub async fn create_battle_recruitment_repository() -> Result<Box<dyn BattleRecruitmentRepository>, PoiseError> {
//         let provider = Self::create_database_provider().await?;
//         Ok(Box::new(database::BattleRecruitmentRepositoryImpl::new(provider)))
//     }

//     /// クエストリポジトリを作成
//     pub async fn create_quest_repository() -> Result<Box<dyn QuestRepository>, PoiseError> {
//         let provider = Self::create_database_provider().await?;
//         Ok(Box::new(database::QuestRepositoryImpl::new(provider)))
//     }

//     /// メッセージテキストリポジトリを作成
//     pub async fn create_message_text_repository() -> Result<Box<dyn MessageTextRepository>, PoiseError> {
//         let provider = Self::create_database_provider().await?;
//         Ok(Box::new(database::MessageTextRepositoryImpl::new(provider)))
//     }

//     /// 環境設定リポジトリを作成
//     pub async fn create_environment_repository() -> Result<Box<dyn EnvironmentRepository>, PoiseError> {
//         let provider = Self::create_database_provider().await?;
//         Ok(Box::new(database::EnvironmentRepositoryImpl::new(provider)))
//     }

//     /// トランザクションマネージャーを作成
//     pub async fn create_transaction_manager() -> Result<database::DatabaseTransactionManager, PoiseError> {
//         let provider = Self::create_database_provider().await?;
//         Ok(database::DatabaseTransactionManager::new(provider))
//     }

//     /// データベースプロバイダーを作成（内部実装を隠蔽）
//     async fn create_database_provider() -> Result<database::DatabaseProvider, PoiseError> {
//         // データベース接続の詳細を内部で処理
//         let conn = crate::repository::database::db_compat::Database::new().await
//             .map_err(|e| PoiseError::from(format!("Failed to connect to database: {}", e)))?;
//         Ok(database::DatabaseProvider::new(conn.conn))
//     }
// }
