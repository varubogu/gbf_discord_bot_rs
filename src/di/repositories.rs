//! リポジトリコンテナ
//!
//! データベースリポジトリ群を保持するコンテナ。
//! Gateway抽象化には依存せず、DatabaseConnectionのみに依存する。

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::repository::database::{
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
    message_text_repository::SeaOrmMessageTextRepository,
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
#[derive(Clone)]
pub struct Repositories {
    // === データベース接続 ===
    /// Guild ロール用DB接続（通常のコマンド実行用、RLS適用）
    pub guild_db: Arc<DatabaseConnection>,
    /// System ロール用DB接続（スケジューラー用、RLS適用なし）
    pub system_db: Arc<DatabaseConnection>,
    /// Global ロール用DB接続（マスターデータ更新用、RLS適用なし）
    pub global_db: Arc<DatabaseConnection>,

    // === 基本リポジトリ ===
    pub battle_recruitments: Arc<SeaOrmBattleRecruitmentsRepository>,
    pub recruitment_participants: Arc<SeaOrmRecruitmentParticipantsRepository>,
    pub guild: Arc<SeaOrmGuildRepository>,
    pub guild_settings: Arc<SeaOrmGuildSettingsRepository>,
    pub guild_channel: Arc<SeaOrmGuildChannelRepository>,
    pub guild_environment: Arc<SeaOrmGuildEnvironmentRepository>,
    pub guild_message_text: Arc<SeaOrmGuildMessageTextRepository>,
    pub guild_quest_disable: Arc<SeaOrmGuildQuestDisableRepository>,
    pub quest: Arc<SeaOrmQuestRepository>,
    pub battle_style: Arc<SeaOrmBattleStyleRepository>,
    pub channel_type: Arc<SeaOrmChannelTypeRepository>,
    pub message_text: Arc<SeaOrmMessageTextRepository>,

    // === スケジュール関連リポジトリ ===
    pub scheduled_task: Arc<SeaOrmScheduledTaskRepository>,
    pub scheduled_task_dismissal: Arc<SeaOrmScheduledTaskDismissalRepository>,
    pub scheduled_task_dissolution: Arc<SeaOrmScheduledTaskDissolutionRepository>,
    pub scheduled_task_cleanup: Arc<SeaOrmScheduledTaskCleanupRepository>,
    pub scheduled_task_recurring: Arc<SeaOrmScheduledTaskRecurringRecruitmentRepository>,
    pub schedule: Arc<SeaOrmScheduleRepository>,
    pub battle_recruitment_schedule: Arc<SeaOrmBattleRecruitmentScheduleRepository>,
    pub battle_recruitment_dismissal: Arc<SeaOrmBattleRecruitmentDismissalRepository>,
    pub battle_recruitment_schedule_dismissal:
        Arc<SeaOrmBattleRecruitmentScheduleDismissalRepository>,
    pub notification: Arc<SeaOrmNotificationRepository>,
    pub notification_rel_battle_recruitment: Arc<SeaOrmNotificationRelBattleRecruitmentRepository>,
    pub notification_rel_event_schedule: Arc<SeaOrmNotificationRelEventScheduleRepository>,

    // === 自動募集関連リポジトリ ===
    pub auto_recruitment: Arc<SeaOrmAutoRecruitmentRepository>,
    pub auto_recruitment_channel: Arc<SeaOrmAutoRecruitmentChannelRepository>,
    pub auto_recruitment_participant: Arc<SeaOrmAutoRecruitmentParticipantRepository>,
    pub auto_recruitment_quest_message: Arc<SeaOrmAutoRecruitmentQuestMessageRepository>,
    pub quest_matching: Arc<SeaOrmQuestMatchingRepository>,
    pub quest_matching_user: Arc<SeaOrmQuestMatchingUserRepository>,
    pub user_desired_quest: Arc<SeaOrmUserDesiredQuestRepository>,
}

impl Repositories {
    /// 新しいRepositoriesを作成する
    pub fn new(
        guild_db: Arc<DatabaseConnection>,
        system_db: Arc<DatabaseConnection>,
        global_db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            guild_db,
            system_db,
            global_db,

            // 基本リポジトリ
            battle_recruitments: Arc::new(SeaOrmBattleRecruitmentsRepository::new()),
            recruitment_participants: Arc::new(SeaOrmRecruitmentParticipantsRepository::new()),
            guild: Arc::new(SeaOrmGuildRepository::new()),
            guild_settings: Arc::new(SeaOrmGuildSettingsRepository::new()),
            guild_channel: Arc::new(SeaOrmGuildChannelRepository::new()),
            guild_environment: Arc::new(SeaOrmGuildEnvironmentRepository::new()),
            guild_message_text: Arc::new(SeaOrmGuildMessageTextRepository::new()),
            guild_quest_disable: Arc::new(SeaOrmGuildQuestDisableRepository::new()),
            quest: Arc::new(SeaOrmQuestRepository::new()),
            battle_style: Arc::new(SeaOrmBattleStyleRepository::new()),
            channel_type: Arc::new(SeaOrmChannelTypeRepository::new()),
            message_text: Arc::new(SeaOrmMessageTextRepository::new()),

            // スケジュール関連リポジトリ
            scheduled_task: Arc::new(SeaOrmScheduledTaskRepository::new()),
            scheduled_task_dismissal: Arc::new(SeaOrmScheduledTaskDismissalRepository::new()),
            scheduled_task_dissolution: Arc::new(SeaOrmScheduledTaskDissolutionRepository::new()),
            scheduled_task_cleanup: Arc::new(SeaOrmScheduledTaskCleanupRepository::new()),
            scheduled_task_recurring: Arc::new(
                SeaOrmScheduledTaskRecurringRecruitmentRepository::new(),
            ),
            schedule: Arc::new(SeaOrmScheduleRepository::new()),
            battle_recruitment_schedule: Arc::new(SeaOrmBattleRecruitmentScheduleRepository::new()),
            battle_recruitment_dismissal: Arc::new(
                SeaOrmBattleRecruitmentDismissalRepository::new(),
            ),
            battle_recruitment_schedule_dismissal: Arc::new(
                SeaOrmBattleRecruitmentScheduleDismissalRepository::new(),
            ),
            notification: Arc::new(SeaOrmNotificationRepository::new()),
            notification_rel_battle_recruitment: Arc::new(
                SeaOrmNotificationRelBattleRecruitmentRepository::new(),
            ),
            notification_rel_event_schedule: Arc::new(
                SeaOrmNotificationRelEventScheduleRepository::new(),
            ),

            // 自動募集関連リポジトリ
            auto_recruitment: Arc::new(SeaOrmAutoRecruitmentRepository::new()),
            auto_recruitment_channel: Arc::new(SeaOrmAutoRecruitmentChannelRepository::new()),
            auto_recruitment_participant: Arc::new(
                SeaOrmAutoRecruitmentParticipantRepository::new(),
            ),
            auto_recruitment_quest_message: Arc::new(
                SeaOrmAutoRecruitmentQuestMessageRepository::new(),
            ),
            quest_matching: Arc::new(SeaOrmQuestMatchingRepository::new()),
            quest_matching_user: Arc::new(SeaOrmQuestMatchingUserRepository::new()),
            user_desired_quest: Arc::new(SeaOrmUserDesiredQuestRepository::new()),
        }
    }

    /// Guild ロール用DB接続を取得（通常のコマンド実行用）
    pub fn guild_db(&self) -> &DatabaseConnection {
        &self.guild_db
    }

    /// System ロール用DB接続を取得（スケジューラー用）
    pub fn system_db(&self) -> &DatabaseConnection {
        &self.system_db
    }

    /// Global ロール用DB接続を取得（マスターデータ更新用）
    pub fn global_db(&self) -> &DatabaseConnection {
        &self.global_db
    }
}
