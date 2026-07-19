use super::auto_matching::run_auto_matching_dispatch;
use super::recurring_recruitment::run_recurring_recruitment_dispatch;
use super::task_timing::is_task_due;
use super::{
    DispatchGuildMessageTextRepository, DispatchMessageTextRepository,
    DispatchParticipantsRepository, DispatchRecruitmentRepository, SchedulerTaskDispatchFacade,
};
use crate::gateway::DiscordGateway;
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::message::MessageService;
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::auto_matching_dispatch_support_service::AutoMatchingDispatchSupportService;
use crate::services::schedule::auto_recruitment_rotation_task_executor::{
    AutoRecruitmentRotationTaskExecutor, StartupRepairResult,
};
use crate::services::schedule::cleanup_task_executor::CleanupTaskExecutor;
use crate::services::schedule::dismissal_task_executor::DismissalTaskExecutor;
use crate::services::schedule::dissolution_task_executor::DissolutionTaskExecutor;
use crate::services::schedule::recurring_recruitment_dispatch_support_service::RecurringRecruitmentDispatchSupportService;
use crate::services::schedule::{
    DismissalManagementService, NotificationManagementService, NotificationService,
    RecruitmentMessageDeletionScheduleService, RecruitmentMessageDeletionTaskExecutor,
    RecruitmentScheduleService, ScheduledTaskDispatchSupportService,
};
use crate::services::timezone_service::TimezoneService;
use crate::types::Result;
use chrono::{Duration, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// task_typeごとの実行分岐・Executor組み立て
impl<R, P, GM, MT> SchedulerTaskDispatchFacade<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    pub(super) async fn run_repair<G: DiscordGateway + Send + Sync + 'static>(
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
    pub(super) async fn run_dispatch_cycle<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &Arc<G>,
    ) -> Result<()> {
        let now = Utc::now();
        let preload_until = now + Duration::seconds(20);

        debug!(to = %preload_until, "タスクをプリロードします");

        let support = ScheduledTaskDispatchSupportService::new(self.repos);
        let tasks = support.find_due_tasks(txn, preload_until).await?;

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
                    self.run_notification_task(txn, gateway, &support, task.id)
                        .await
                }
                2 => {
                    self.run_dissolution_task(txn, gateway, &support, task.id)
                        .await
                }
                3 => self.run_cleanup_task(txn, task.id).await,
                4 => {
                    self.run_recurring_recruitment_task(txn, db_conn, gateway, &support, task.id)
                        .await
                }
                5 => {
                    self.run_dismissal_task(txn, gateway, &support, task.id)
                        .await
                }
                6 => {
                    self.run_rotation_task(txn, gateway, &support, task.id)
                        .await
                }
                7 => {
                    self.run_auto_matching_task(txn, db_conn, gateway, &support, task.id)
                        .await
                }
                8 => {
                    self.run_message_deletion_task(txn, gateway, &support, task.id)
                        .await
                }
                _ => {
                    warn!(
                        task_id = task.id,
                        task_type = task.task_type,
                        "不明なタスクタイプです"
                    );
                    support.mark_failed(txn, task.id).await;
                }
            }
        }

        Ok(())
    }

    /// task_type=1: 通知タスク
    async fn run_notification_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "通知タスクを実行します");

        let notification_service = NotificationService::new(
            Arc::clone(gateway),
            Arc::clone(&self.recruitment_repo),
            self.repos.notification,
            self.repos.notification_rel_battle_recruitment,
            self.repos.guild_settings,
            self.repos.recruitment_participants,
            MessageService::new(self.repos.guild_message_text, self.repos.message_text),
        );

        match support.find_notification_for_task(txn, task_id).await {
            Ok(Some(notification)) => {
                match notification_service
                    .send_single_notification(txn, &notification)
                    .await
                {
                    Ok(_) => {
                        if let Err(e) = support.mark_succeeded(txn, task_id).await {
                            error!(task_id, error = %e, "タスクの完了マークに失敗しました");
                        }
                        info!(task_id, "通知タスクを実行しました");
                    }
                    Err(e) => {
                        error!(task_id, error = %e, "通知の送信中にエラーが発生しました");
                        support.mark_failed(txn, task_id).await;
                    }
                }
            }
            Ok(None) => {
                warn!(
                    task_id,
                    "通知が見つかりません（データ不整合）。タスクを警告付き完了としてマークします"
                );
                if let Err(e) = support.mark_succeeded_with_warning(txn, task_id).await {
                    error!(task_id, error = %e, "タスクの完了マークに失敗しました");
                }
            }
            Err(e) => {
                error!(task_id, error = %e, "通知の取得に失敗しました");
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=2: 解散タスク
    async fn run_dissolution_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "解散タスクを実行します");

        let executor = DissolutionTaskExecutor::new(
            Arc::new(self.repos.scheduled_task),
            Arc::new(self.repos.scheduled_task_dissolution),
            Arc::clone(&self.recruitment_repo),
            Arc::clone(&self.participants_repo),
            Arc::clone(&self.message_service),
            Arc::new(self.repos.guild_settings),
            self.repos.quest,
        );

        match executor.execute(txn, gateway.as_ref(), task_id).await {
            Ok(result) => {
                info!(task_id, result = ?result, "解散タスクを実行しました");
            }
            Err(e) => {
                error!(task_id, error = %e, "解散タスクの実行中にエラーが発生しました");
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=3: データクリーンアップタスク
    ///
    /// 失敗マークは `CleanupTaskExecutor` 内部で完結するため、ここでは行わない。
    async fn run_cleanup_task(&self, txn: &DatabaseTransaction, task_id: i32) {
        info!(task_id, "データクリーンアップタスクを実行します");

        let executor = CleanupTaskExecutor::new(self.repos);
        match executor.execute(txn, task_id).await {
            Ok(result) => {
                info!(task_id, result = ?result, "データクリーンアップタスクを実行しました");
            }
            Err(e) => {
                error!(
                    task_id,
                    error = %e,
                    "データクリーンアップタスクの実行中にエラーが発生しました"
                );
            }
        }
    }

    /// task_type=4: 定期募集タスク
    async fn run_recurring_recruitment_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "定期募集タスクを実行します");

        let schedule_service = Arc::new(RecruitmentScheduleService::new());

        let dispatch_support = RecurringRecruitmentDispatchSupportService::new(
            self.repos.scheduled_task,
            self.repos.scheduled_task_recurring,
            self.repos.battle_recruitment_schedule,
            Arc::clone(&schedule_service),
        );

        let role_service = RoleNotificationService::new(
            self.repos.all_recruitment_notification_roles,
            self.repos.quest_recruitment_notification_roles,
        );

        let timezone_service = TimezoneService::new(self.repos.guild_settings);
        let guild_env_service = GuildEnvironmentService::new(self.repos.guild_environment);

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

        let message_deletion_schedule_service = RecruitmentMessageDeletionScheduleService::new(
            self.repos.guild_environment,
            self.repos.environment,
            self.repos.scheduled_task,
            self.repos.scheduled_task_recruitment_message_deletion,
        );

        let recruitment_creation_service = RecruitmentCreationService::new(
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
        );

        match run_recurring_recruitment_dispatch(
            txn,
            db_conn,
            gateway.as_ref(),
            &dispatch_support,
            &schedule_service,
            &recruitment_creation_service,
            task_id,
        )
        .await
        {
            Ok(result) => {
                info!(task_id, result = ?result, "定期募集タスクを実行しました");
            }
            Err(e) => {
                error!(task_id, error = %e, "定期募集タスクの実行中にエラーが発生しました");
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=5: 解散（人数不足）タスク
    async fn run_dismissal_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "解散（人数不足）タスクを実行します");

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

        match executor.execute(txn, gateway.as_ref(), task_id).await {
            Ok(result) => {
                info!(task_id, result = ?result, "解散（人数不足）タスクを実行しました");
            }
            Err(e) => {
                error!(
                    task_id,
                    error = %e,
                    "解散（人数不足）タスクの実行中にエラーが発生しました"
                );
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=6: 自動募集日付ローテーションタスク
    async fn run_rotation_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "自動募集日付ローテーションタスクを実行します");

        let executor = AutoRecruitmentRotationTaskExecutor::new(
            Arc::new(self.repos.scheduled_task),
            Arc::new(self.repos.auto_recruitment_channel),
            self.repos.auto_recruitment,
        );

        match executor.execute(txn, gateway.as_ref(), task_id).await {
            Ok(result) => {
                info!(task_id, result = ?result, "自動募集日付ローテーションタスクを実行しました");
            }
            Err(e) => {
                error!(
                    task_id,
                    error = %e,
                    "自動募集日付ローテーションタスクの実行中にエラーが発生しました"
                );
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=7: 自動マッチングタスク
    async fn run_auto_matching_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "自動マッチングタスクを実行します");

        let role_service = RoleNotificationService::new(
            self.repos.all_recruitment_notification_roles,
            self.repos.quest_recruitment_notification_roles,
        );

        let timezone_service = TimezoneService::new(self.repos.guild_settings);
        let guild_env_service = GuildEnvironmentService::new(self.repos.guild_environment);

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

        let message_deletion_schedule_service = RecruitmentMessageDeletionScheduleService::new(
            self.repos.guild_environment,
            self.repos.environment,
            self.repos.scheduled_task,
            self.repos.scheduled_task_recruitment_message_deletion,
        );

        let recruitment_creation_service = RecruitmentCreationService::new(
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
        );

        let matching_service = PeriodicMatchingService::new(
            self.repos.auto_recruitment_participant,
            self.repos.user_desired_quest,
            self.repos.quest_matching,
            self.repos.quest_matching_user,
            self.repos.quest,
            self.repos.auto_recruitment_match_rule,
            self.repos.auto_recruitment_match_rule_quota,
        );

        let dispatch_support = AutoMatchingDispatchSupportService::new(
            self.repos.scheduled_task,
            self.repos.auto_recruitment,
            self.repos.quest_matching,
            self.repos.quest_matching_user,
            self.repos.quest,
        );

        match run_auto_matching_dispatch(
            txn,
            db_conn,
            gateway.as_ref(),
            &dispatch_support,
            &matching_service,
            &recruitment_creation_service,
            task_id,
        )
        .await
        {
            Ok(result) => {
                info!(task_id, result = ?result, "自動マッチングタスクを実行しました");
            }
            Err(e) => {
                error!(task_id, error = %e, "自動マッチングタスクの実行中にエラーが発生しました");
                support.mark_failed(txn, task_id).await;
            }
        }
    }

    /// task_type=8: 募集投稿削除タスク
    async fn run_message_deletion_task<G: DiscordGateway + Send + Sync + 'static>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &Arc<G>,
        support: &ScheduledTaskDispatchSupportService,
        task_id: i32,
    ) {
        info!(task_id, "募集投稿削除タスクを実行します");

        let executor = RecruitmentMessageDeletionTaskExecutor::new(
            Arc::new(self.repos.scheduled_task),
            Arc::new(self.repos.scheduled_task_recruitment_message_deletion),
            Arc::clone(&self.recruitment_repo),
        );

        match executor.execute(txn, gateway.as_ref(), task_id).await {
            Ok(result) => {
                info!(task_id, result = ?result, "募集投稿削除タスクを実行しました");
            }
            Err(e) => {
                error!(task_id, error = %e, "募集投稿削除タスクの実行中にエラーが発生しました");
                support.mark_failed(txn, task_id).await;
            }
        }
    }
}
