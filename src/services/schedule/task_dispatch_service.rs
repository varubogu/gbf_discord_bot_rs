use crate::di::Repositories;
use crate::gateway::DiscordGateway;
use crate::repository::schedule::{
    NotificationRepository as NotificationRepositoryTrait, ScheduledTaskRepository,
};
use crate::repository::{
    BattleRecruitmentsRepository, GuildMessageTextRepository, MessageTextRepository,
    RecruitmentParticipantsRepository,
};
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::message::MessageService;
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::auto_recruitment_rotation_task_executor::{
    AutoRecruitmentRotationTaskExecutor, StartupRepairResult,
};
use crate::services::schedule::cleanup_task_executor::CleanupTaskExecutor;
use crate::services::schedule::dismissal_task_executor::DismissalTaskExecutor;
use crate::services::schedule::dissolution_task_executor::DissolutionTaskExecutor;
use crate::services::schedule::recurring_recruitment_task_executor::RecurringRecruitmentTaskExecutor;
use crate::services::schedule::{
    DismissalManagementService, NotificationManagementService, NotificationService,
    RecruitmentMessageDeletionScheduleService, RecruitmentMessageDeletionTaskExecutor,
    RecruitmentScheduleService,
};
use crate::services::timezone_service::TimezoneService;
use crate::types::Result;
use chrono::{Duration, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// TaskDispatchService向け募集リポジトリ境界
pub trait DispatchRecruitmentRepository: BattleRecruitmentsRepository + 'static {}

impl<T> DispatchRecruitmentRepository for T where T: BattleRecruitmentsRepository + 'static {}

/// TaskDispatchService向け参加者リポジトリ境界
pub trait DispatchParticipantsRepository: RecruitmentParticipantsRepository + 'static {}

impl<T> DispatchParticipantsRepository for T where T: RecruitmentParticipantsRepository + 'static {}

/// TaskDispatchService向けギルドメッセージリポジトリ境界
pub trait DispatchGuildMessageTextRepository:
    GuildMessageTextRepository + Clone + Send + Sync + 'static
{
}

impl<T> DispatchGuildMessageTextRepository for T where
    T: GuildMessageTextRepository + Clone + Send + Sync + 'static
{
}

/// TaskDispatchService向けメッセージリポジトリ境界
pub trait DispatchMessageTextRepository:
    MessageTextRepository + Clone + Send + Sync + 'static
{
}

impl<T> DispatchMessageTextRepository for T where
    T: MessageTextRepository + Clone + Send + Sync + 'static
{
}

/// スケジュールタスクの実行ディスパッチサービス
///
/// - task_typeごとの実行分岐
/// - Executorの依存組み立て
/// - タスク結果ステータス更新
pub struct TaskDispatchService<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    recruitment_repo: Arc<R>,
    participants_repo: Arc<P>,
    message_service: Arc<MessageService<GM, MT>>,
    repos: Repositories,
}

impl<R, P, GM, MT> TaskDispatchService<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    pub fn new(
        recruitment_repo: Arc<R>,
        participants_repo: Arc<P>,
        message_service: Arc<MessageService<GM, MT>>,
        repos: Repositories,
    ) -> Self {
        Self {
            recruitment_repo,
            participants_repo,
            message_service,
            repos,
        }
    }

    /// 起動時の自動募集日時チャンネル補正を実行
    pub async fn repair_auto_recruitment_channels_on_startup<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
    ) -> Result<StartupRepairResult> {
        let executor = AutoRecruitmentRotationTaskExecutor::new(
            Arc::new(self.repos.scheduled_task),
            Arc::new(self.repos.auto_recruitment_channel),
            self.repos.auto_recruitment,
        );

        executor.repair_on_startup(txn, gateway.as_ref()).await
    }

    /// 期限到達済みタスクをプリロードして実行
    pub async fn dispatch_due_tasks<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &Arc<G>,
    ) -> Result<()> {
        let now = Utc::now();
        let preload_until = now + Duration::seconds(20);

        debug!(to = %preload_until, "タスクをプリロードします");

        let tasks: Vec<crate::models::entities::worker::scheduled_tasks::Model> = self
            .repos
            .scheduled_task
            .find_pending_to(txn, preload_until)
            .await?;

        if tasks.is_empty() {
            debug!("プリロード対象のタスクはありません");
            return Ok(());
        }

        info!(tasks = tasks.len(), "タスクをプリロードしました");

        for task in tasks {
            if !is_task_due(task.schedule_datetime, now) {
                debug!(
                    task_id = task.id,
                    schedule_datetime = %task.schedule_datetime,
                    "タスクはまだ実行時刻に達していません"
                );
                continue;
            }

            match task.task_type {
                1 => {
                    info!(task_id = task.id, "通知タスクを実行します");

                    let notification_service = NotificationService::new(
                        Arc::clone(gateway),
                        Arc::clone(&self.recruitment_repo),
                        self.repos.notification,
                        self.repos.notification_rel_battle_recruitment,
                        self.repos.guild_settings,
                        self.repos.recruitment_participants,
                        MessageService::new(self.repos.guild_message_text, self.repos.message_text),
                    );

                    match self.repos.notification.find_by_task_id(txn, task.id).await {
                        Ok(Some(notification)) => {
                            match notification_service
                                .send_single_notification(txn, &notification)
                                .await
                            {
                                Ok(_) => {
                                    if let Err(e) = self
                                        .repos
                                        .scheduled_task
                                        .mark_as_succeeded(txn, task.id)
                                        .await
                                    {
                                        error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                                    }
                                    info!(task_id = task.id, "通知タスクを実行しました");
                                }
                                Err(e) => {
                                    error!(task_id = task.id, error = %e, "通知の送信中にエラーが発生しました");
                                    self.mark_task_as_failed(txn, task.id).await;
                                }
                            }
                        }
                        Ok(None) => {
                            warn!(
                                task_id = task.id,
                                "通知が見つかりません（データ不整合）。タスクを警告付き完了としてマークします"
                            );
                            if let Err(e) = self
                                .repos
                                .scheduled_task
                                .mark_as_succeeded_with_warning(txn, task.id)
                                .await
                            {
                                error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                            }
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "通知の取得に失敗しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                2 => {
                    info!(task_id = task.id, "解散タスクを実行します");
                    let executor = DissolutionTaskExecutor::new(
                        Arc::new(self.repos.scheduled_task),
                        Arc::new(self.repos.scheduled_task_dissolution),
                        Arc::clone(&self.recruitment_repo),
                        Arc::clone(&self.participants_repo),
                        Arc::clone(&self.message_service),
                        Arc::new(self.repos.guild_settings),
                        self.repos.quest,
                    );

                    match executor.execute(txn, gateway.as_ref(), task.id).await {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "解散タスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "解散タスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                3 => {
                    info!(task_id = task.id, "データクリーンアップタスクを実行します");
                    let executor = CleanupTaskExecutor::new(self.repos);
                    match executor.execute(txn, task.id).await {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "データクリーンアップタスクを実行しました");
                        }
                        Err(e) => {
                            error!(
                                task_id = task.id,
                                error = %e,
                                "データクリーンアップタスクの実行中にエラーが発生しました"
                            );
                        }
                    }
                }
                4 => {
                    info!(task_id = task.id, "定期募集タスクを実行します");

                    let schedule_service = Arc::new(RecruitmentScheduleService::new());

                    let role_service = RoleNotificationService::new(
                        self.repos.all_recruitment_notification_roles,
                        self.repos.quest_recruitment_notification_roles,
                    );

                    let timezone_service = TimezoneService::new(self.repos.guild_settings);
                    let guild_env_service =
                        GuildEnvironmentService::new(self.repos.guild_environment);

                    let notification_management_service = NotificationManagementService::new(
                        self.repos.notification,
                        self.repos.notification_rel_battle_recruitment,
                        self.repos.scheduled_task,
                    );

                    let dismissal_service = DismissalManagementService::new(
                        self.repos.battle_recruitment_dismissal,
                        self.repos.scheduled_task,
                        self.repos.scheduled_task_dismissal,
                    );

                    let message_deletion_schedule_service =
                        RecruitmentMessageDeletionScheduleService::new(
                            self.repos.guild_environment,
                            self.repos.environment,
                            self.repos.scheduled_task,
                            self.repos.scheduled_task_recruitment_message_deletion,
                        );

                    let recruitment_creation_service = Arc::new(RecruitmentCreationService::new(
                        self.repos.guild_channel,
                        self.repos.quest,
                        self.repos.battle_style,
                        role_service,
                        timezone_service,
                        guild_env_service,
                        self.repos.battle_recruitment_schedule_dismissal,
                        MessageService::new(self.repos.guild_message_text, self.repos.message_text),
                        notification_management_service,
                        dismissal_service,
                        self.repos.battle_recruitments,
                        message_deletion_schedule_service,
                    ));

                    let executor = RecurringRecruitmentTaskExecutor::new(
                        self.repos.scheduled_task,
                        self.repos.scheduled_task_recurring,
                        self.repos.battle_recruitment_schedule,
                        schedule_service,
                        recruitment_creation_service,
                    );

                    match executor
                        .execute(txn, db_conn, gateway.as_ref(), task.id)
                        .await
                    {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "定期募集タスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "定期募集タスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                5 => {
                    info!(task_id = task.id, "解散（人数不足）タスクを実行します");

                    let executor = DismissalTaskExecutor::new(
                        Arc::clone(&self.message_service),
                        Arc::new(self.repos.scheduled_task),
                        Arc::new(self.repos.scheduled_task_dismissal),
                        Arc::new(self.repos.battle_recruitment_dismissal),
                        Arc::clone(&self.recruitment_repo),
                        Arc::clone(&self.participants_repo),
                        Arc::new(self.repos.quest),
                        Arc::new(self.repos.guild_settings),
                        Arc::new(self.repos.notification),
                        Arc::new(self.repos.notification_rel_battle_recruitment),
                    );

                    match executor.execute(txn, gateway.as_ref(), task.id).await {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "解散（人数不足）タスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "解散（人数不足）タスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                6 => {
                    info!(
                        task_id = task.id,
                        "自動募集日付ローテーションタスクを実行します"
                    );

                    let executor = AutoRecruitmentRotationTaskExecutor::new(
                        Arc::new(self.repos.scheduled_task),
                        Arc::new(self.repos.auto_recruitment_channel),
                        self.repos.auto_recruitment,
                    );

                    match executor.execute(txn, gateway.as_ref(), task.id).await {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "自動募集日付ローテーションタスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "自動募集日付ローテーションタスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                7 => {
                    info!(task_id = task.id, "自動マッチングタスクを実行します");
                    use crate::services::schedule::auto_matching_task_executor::AutoMatchingTaskExecutor;

                    let role_service = RoleNotificationService::new(
                        self.repos.all_recruitment_notification_roles,
                        self.repos.quest_recruitment_notification_roles,
                    );

                    let timezone_service = TimezoneService::new(self.repos.guild_settings);
                    let guild_env_service =
                        GuildEnvironmentService::new(self.repos.guild_environment);

                    let notification_management_service = NotificationManagementService::new(
                        self.repos.notification,
                        self.repos.notification_rel_battle_recruitment,
                        self.repos.scheduled_task,
                    );

                    let dismissal_service = DismissalManagementService::new(
                        self.repos.battle_recruitment_dismissal,
                        self.repos.scheduled_task,
                        self.repos.scheduled_task_dismissal,
                    );

                    let message_deletion_schedule_service =
                        RecruitmentMessageDeletionScheduleService::new(
                            self.repos.guild_environment,
                            self.repos.environment,
                            self.repos.scheduled_task,
                            self.repos.scheduled_task_recruitment_message_deletion,
                        );

                    let recruitment_creation_service = Arc::new(RecruitmentCreationService::new(
                        self.repos.guild_channel,
                        self.repos.quest,
                        self.repos.battle_style,
                        role_service,
                        timezone_service,
                        guild_env_service,
                        self.repos.battle_recruitment_schedule_dismissal,
                        MessageService::new(self.repos.guild_message_text, self.repos.message_text),
                        notification_management_service,
                        dismissal_service,
                        self.repos.battle_recruitments,
                        message_deletion_schedule_service,
                    ));

                    let matching_service = PeriodicMatchingService::new(
                        self.repos.auto_recruitment_participant,
                        self.repos.user_desired_quest,
                        self.repos.quest_matching,
                        self.repos.quest_matching_user,
                        self.repos.quest,
                        self.repos.auto_recruitment_match_rule,
                        self.repos.auto_recruitment_match_rule_quota,
                    );

                    let executor = AutoMatchingTaskExecutor::new(
                        Arc::new(self.repos.scheduled_task),
                        recruitment_creation_service,
                        matching_service,
                        self.repos.auto_recruitment,
                        self.repos.quest_matching_user,
                        self.repos.quest_matching,
                        self.repos.quest,
                    );

                    match executor
                        .execute(txn, db_conn, gateway.as_ref(), task.id)
                        .await
                    {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "自動マッチングタスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "自動マッチングタスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                8 => {
                    info!(task_id = task.id, "募集投稿削除タスクを実行します");

                    let executor = RecruitmentMessageDeletionTaskExecutor::new(
                        Arc::new(self.repos.scheduled_task),
                        Arc::new(self.repos.scheduled_task_recruitment_message_deletion),
                        Arc::clone(&self.recruitment_repo),
                    );

                    match executor.execute(txn, gateway.as_ref(), task.id).await {
                        Ok(result) => {
                            info!(task_id = task.id, result = ?result, "募集投稿削除タスクを実行しました");
                        }
                        Err(e) => {
                            error!(task_id = task.id, error = %e, "募集投稿削除タスクの実行中にエラーが発生しました");
                            self.mark_task_as_failed(txn, task.id).await;
                        }
                    }
                }
                _ => {
                    warn!(
                        task_id = task.id,
                        task_type = task.task_type,
                        "不明なタスクタイプです"
                    );
                    self.mark_task_as_failed(txn, task.id).await;
                }
            }
        }

        Ok(())
    }

    async fn mark_task_as_failed(&self, txn: &DatabaseTransaction, task_id: i32) {
        if let Err(mark_err) = self.repos.scheduled_task.mark_as_failed(txn, task_id).await {
            error!(
                task_id = task_id,
                error = %mark_err,
                "タスクの失敗マークに失敗しました"
            );
        }
    }
}

fn is_task_due(task_schedule_datetime: chrono::DateTime<Utc>, now: chrono::DateTime<Utc>) -> bool {
    task_schedule_datetime <= now
}

#[cfg(test)]
mod tests {
    use super::is_task_due;
    use chrono::{Duration, Utc};

    #[test]
    fn is_due_when_schedule_is_past_or_now() {
        let now = Utc::now();
        assert!(is_task_due(now - Duration::seconds(1), now));
        assert!(is_task_due(now, now));
    }

    #[test]
    fn is_not_due_when_schedule_is_future() {
        let now = Utc::now();
        assert!(!is_task_due(now + Duration::seconds(1), now));
    }
}
