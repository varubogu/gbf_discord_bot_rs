use crate::di::Repositories;
use crate::gateway::DiscordGateway;
use crate::repository::schedule::ScheduledTaskRepository;
use crate::repository::{
    BattleRecruitmentsRepository, GuildMessageTextRepository, MessageTextRepository,
    RecruitmentParticipantsRepository,
};
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::message::MessageService;
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::dismissal_task_executor::DismissalTaskExecutor;
use crate::services::schedule::dissolution_task_executor::DissolutionTaskExecutor;
use crate::services::schedule::recurring_recruitment_task_executor::RecurringRecruitmentTaskExecutor;
use crate::services::schedule::{
    DismissalManagementService, NotificationManagementService, NotificationService,
    RecruitmentScheduleService,
};
use crate::services::timezone_service::TimezoneService;
use crate::types::Result;
use chrono::{Duration, Utc};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, warn};

/// スケジューラーマネージャー
///
/// tokio-cron-schedulerを使用して、定期的にタスクをプリロードし、実行する
pub struct SchedulerManager<G, R, P, GM, MT>
where
    G: DiscordGateway + Send + Sync + 'static,
    R: BattleRecruitmentsRepository + 'static,
    P: RecruitmentParticipantsRepository + 'static,
    GM: GuildMessageTextRepository + Clone + Send + Sync + 'static,
    MT: MessageTextRepository + Clone + Send + Sync + 'static,
{
    scheduler: JobScheduler,
    gateway: Arc<G>,
    recruitment_repo: Arc<R>,
    participants_repo: Arc<P>,
    message_service: Arc<MessageService<GM, MT>>,
    repos: Repositories,
}

impl<G, R, P, GM, MT> SchedulerManager<G, R, P, GM, MT>
where
    G: DiscordGateway + Send + Sync + 'static,
    R: BattleRecruitmentsRepository + 'static,
    P: RecruitmentParticipantsRepository + 'static,
    GM: GuildMessageTextRepository + Clone + Send + Sync + 'static,
    MT: MessageTextRepository + Clone + Send + Sync + 'static,
{
    /// 新しいSchedulerManagerを作成
    ///
    /// # Arguments
    /// * `gateway` - Discord Gateway（トレイト境界による抽象化）
    /// * `recruitment_repo` - 募集リポジトリ
    /// * `participants_repo` - 参加者リポジトリ
    /// * `message_service` - メッセージサービス
    /// * `repos` - リポジトリコンテナ
    pub async fn new(
        gateway: Arc<G>,
        recruitment_repo: Arc<R>,
        participants_repo: Arc<P>,
        message_service: Arc<MessageService<GM, MT>>,
        repos: Repositories,
    ) -> Result<Self> {
        let scheduler = JobScheduler::new().await.map_err(|e| {
            crate::types::AppError::Generic(format!("JobScheduler creation failed: {e}"))
        })?;

        Ok(Self {
            scheduler,
            gateway,
            recruitment_repo,
            participants_repo,
            message_service,
            repos,
        })
    }

    /// スケジューラーを開始
    ///
    /// 10秒間隔でタスクをプリロードするジョブを登録し、スケジューラーを起動する
    ///
    /// # Arguments
    /// * `db` - データベース接続
    pub async fn start(&mut self, db: Arc<DatabaseConnection>) -> Result<()> {
        info!("SchedulerManagerを起動します");

        // 起動時に自動募集日時チャンネルを補正（失敗しても起動継続）
        if let Err(e) =
            Self::repair_auto_recruitment_channels_on_startup(&db, &self.gateway, &self.repos).await
        {
            error!(
                error = %e,
                "起動時の自動募集日時チャンネル補正に失敗しました。スケジューラー起動は継続します"
            );
        }

        // 10秒間隔でタスクをプリロードするジョブを作成
        let gateway = Arc::clone(&self.gateway);
        let recruitment_repo = Arc::clone(&self.recruitment_repo);
        let participants_repo = Arc::clone(&self.participants_repo);
        let message_service = Arc::clone(&self.message_service);
        let repos = self.repos;

        let job = Job::new_async("*/10 * * * * *", move |_uuid, _lock| {
            let db = Arc::clone(&db);
            let gateway = Arc::clone(&gateway);
            let recruitment_repo = Arc::clone(&recruitment_repo);
            let participants_repo = Arc::clone(&participants_repo);
            let message_service = Arc::clone(&message_service);

            Box::pin(async move {
                if let Err(e) = Self::preload_and_execute_tasks(
                    &db,
                    &gateway,
                    &recruitment_repo,
                    &participants_repo,
                    &message_service,
                    &repos,
                )
                .await
                {
                    error!(error = %e, "タスクのプリロード・実行中にエラーが発生しました");
                }
            })
        })
        .map_err(|e| crate::types::AppError::Generic(format!("Job creation failed: {e}")))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| crate::types::AppError::Generic(format!("Job add failed: {e}")))?;
        self.scheduler
            .start()
            .await
            .map_err(|e| crate::types::AppError::Generic(format!("Scheduler start failed: {e}")))?;

        info!("SchedulerManagerを起動しました");
        Ok(())
    }

    /// スケジューラーを停止
    pub async fn stop(&mut self) -> Result<()> {
        info!("SchedulerManagerを停止します");
        self.scheduler.shutdown().await.map_err(|e| {
            crate::types::AppError::Generic(format!("Scheduler shutdown failed: {e}"))
        })?;
        info!("SchedulerManagerを停止しました");
        Ok(())
    }

    /// 起動時の自動募集日時チャンネル補正を実行
    async fn repair_auto_recruitment_channels_on_startup(
        db: &Arc<DatabaseConnection>,
        gateway: &Arc<G>,
        repos: &Repositories,
    ) -> Result<()> {
        use crate::services::schedule::auto_recruitment_rotation_task_executor::AutoRecruitmentRotationTaskExecutor;

        let txn = db.begin().await?;
        let executor = AutoRecruitmentRotationTaskExecutor::new(
            Arc::new(repos.scheduled_task),
            Arc::new(repos.auto_recruitment_channel),
            repos.auto_recruitment,
        );

        let result = executor.repair_on_startup(&txn, gateway.as_ref()).await;
        match result {
            Ok(stats) => {
                txn.commit().await?;
                info!(
                    total_guilds = stats.total_guilds,
                    repaired_guilds = stats.repaired_guilds,
                    failed_guilds = stats.failed_guilds,
                    created_channels = stats.created_channels,
                    rotated_channels = stats.rotated_channels,
                    "起動時の自動募集日時チャンネル補正が完了しました"
                );
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = txn.rollback().await {
                    error!(
                        error = %rollback_err,
                        "起動時補正トランザクションのロールバックに失敗しました"
                    );
                }
                Err(e)
            }
        }
    }

    /// タスクをプリロードして実行
    ///
    /// 現在時刻から20秒先までの未実行タスクを取得し、実行時刻に達しているものを実行する
    async fn preload_and_execute_tasks(
        db: &Arc<DatabaseConnection>,
        gateway: &Arc<G>,
        recruitment_repo: &Arc<R>,
        participants_repo: &Arc<P>,
        message_service: &Arc<MessageService<GM, MT>>,
        repos: &Repositories,
    ) -> Result<()> {
        let now = Utc::now();
        let preload_until = now + Duration::seconds(20);

        debug!(
            to = %preload_until,
            "タスクをプリロードします"
        );

        // トランザクション開始
        let txn = db.begin().await?;

        // 保留中タスクを取得（scheduled_tasks）
        // execution_status = pending で絞り込んでいるため、過去の保留中タスクも取得される
        let tasks: Vec<crate::models::entities::worker::scheduled_tasks::Model> = repos
            .scheduled_task
            .find_pending_to(&txn, preload_until)
            .await?;

        if tasks.is_empty() {
            debug!("プリロード対象のタスクはありません");
            txn.commit().await?;
            return Ok(());
        }

        info!(tasks = tasks.len(), "タスクをプリロードしました");

        // scheduled_tasksを実行

        use crate::repository::schedule::NotificationRepository as NotificationRepositoryTrait;

        // Gateway経由でNotificationServiceを作成
        let notification_service = NotificationService::new(
            Arc::clone(gateway),
            Arc::clone(recruitment_repo),
            repos.notification,
            repos.notification_rel_battle_recruitment,
            repos.guild_settings,
            repos.recruitment_participants,
            MessageService::new(repos.guild_message_text, repos.message_text),
        );

        for task in tasks {
            if task.schedule_datetime <= now {
                // 実行時刻に達している場合、タスクを実行
                match task.task_type {
                    1 => {
                        // Notification
                        info!(task_id = task.id, "通知タスクを実行します");

                        // task_idからnotificationsテーブルを検索
                        match repos.notification.find_by_task_id(&txn, task.id).await {
                            Ok(Some(notification)) => {
                                // 通知を送信
                                match notification_service
                                    .send_single_notification(&txn, &notification)
                                    .await
                                {
                                    Ok(_) => {
                                        // タスクを正常終了としてマーク
                                        if let Err(e) = repos
                                            .scheduled_task
                                            .mark_as_succeeded(&txn, task.id)
                                            .await
                                        {
                                            error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                                        }
                                        info!(task_id = task.id, "通知タスクを実行しました");
                                    }
                                    Err(e) => {
                                        error!(task_id = task.id, error = %e, "通知の送信中にエラーが発生しました");
                                        if let Err(mark_err) =
                                            repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                        {
                                            error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                        }
                                    }
                                }
                            }
                            Ok(None) => {
                                // データ不整合：notificationsテーブルに通知がない
                                // このタスクは実行不可能なため、警告付き完了としてマークして次回以降スキップ
                                warn!(
                                    task_id = task.id,
                                    "通知が見つかりません（データ不整合）。タスクを警告付き完了としてマークします"
                                );
                                if let Err(e) = repos
                                    .scheduled_task
                                    .mark_as_succeeded_with_warning(&txn, task.id)
                                    .await
                                {
                                    error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                                }
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "通知の取得に失敗しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                            }
                        }
                    }
                    2 => {
                        // Dissolution
                        info!(task_id = task.id, "解散タスクを実行します");
                        let executor = DissolutionTaskExecutor::new(
                            Arc::new(repos.scheduled_task),
                            Arc::new(repos.scheduled_task_dissolution),
                            Arc::clone(recruitment_repo),
                            Arc::clone(participants_repo),
                            Arc::clone(message_service),
                            Arc::new(repos.guild_settings),
                            repos.quest,
                        );

                        match executor.execute(&txn, gateway.as_ref(), task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "解散タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "解散タスクの実行中にエラーが発生しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    3 => {
                        // DataCleanup
                        warn!(task_id = task.id, "DataCleanupタスクは未実装です");
                        if let Err(mark_err) =
                            repos.scheduled_task.mark_as_failed(&txn, task.id).await
                        {
                            error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                        }
                        // TODO: CleanupTaskExecutorを実装
                    }
                    4 => {
                        // RecurringRecruitment
                        info!(task_id = task.id, "定期募集タスクを実行します");

                        let schedule_service = Arc::new(RecruitmentScheduleService::new());

                        let role_service = RoleNotificationService::new(
                            repos.all_recruitment_notification_roles,
                            repos.quest_recruitment_notification_roles,
                        );

                        let timezone_service = TimezoneService::new(repos.guild_settings);

                        let guild_env_service =
                            GuildEnvironmentService::new(repos.guild_environment);

                        let notification_management_service = NotificationManagementService::new(
                            repos.notification,
                            repos.notification_rel_battle_recruitment,
                            repos.scheduled_task,
                        );

                        let dismissal_service = DismissalManagementService::new(
                            repos.battle_recruitment_dismissal,
                            repos.scheduled_task,
                            repos.scheduled_task_dismissal,
                        );

                        let recruitment_creation_service =
                            Arc::new(RecruitmentCreationService::new(
                                repos.guild_channel,
                                repos.quest,
                                repos.battle_style,
                                role_service,
                                timezone_service,
                                guild_env_service,
                                repos.battle_recruitment_schedule_dismissal,
                                MessageService::new(repos.guild_message_text, repos.message_text),
                                notification_management_service,
                                dismissal_service,
                                repos.battle_recruitments,
                            ));

                        let executor = RecurringRecruitmentTaskExecutor::new(
                            repos.scheduled_task,
                            repos.scheduled_task_recurring,
                            repos.battle_recruitment_schedule,
                            schedule_service,
                            recruitment_creation_service,
                        );

                        match executor.execute(&txn, db, gateway.as_ref(), task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "定期募集タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "定期募集タスクの実行中にエラーが発生しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    5 => {
                        // Dismissal
                        info!(task_id = task.id, "解散（人数不足）タスクを実行します");

                        let executor = DismissalTaskExecutor::new(
                            Arc::clone(message_service),
                            Arc::new(repos.scheduled_task),
                            Arc::new(repos.scheduled_task_dismissal),
                            Arc::new(repos.battle_recruitment_dismissal),
                            Arc::clone(recruitment_repo),
                            Arc::clone(participants_repo),
                            Arc::new(repos.quest),
                            Arc::new(repos.guild_settings),
                            Arc::new(repos.notification),
                            Arc::new(repos.notification_rel_battle_recruitment),
                        );

                        match executor.execute(&txn, gateway.as_ref(), task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "解散（人数不足）タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "解散（人数不足）タスクの実行中にエラーが発生しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    6 => {
                        // AutoRecruitmentRotation
                        info!(
                            task_id = task.id,
                            "自動募集日付ローテーションタスクを実行します"
                        );
                        use crate::services::schedule::auto_recruitment_rotation_task_executor::AutoRecruitmentRotationTaskExecutor;

                        let executor = AutoRecruitmentRotationTaskExecutor::new(
                            Arc::new(repos.scheduled_task),
                            Arc::new(repos.auto_recruitment_channel),
                            repos.auto_recruitment,
                        );

                        match executor.execute(&txn, gateway.as_ref(), task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "自動募集日付ローテーションタスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "自動募集日付ローテーションタスクの実行中にエラーが発生しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    7 => {
                        // AutoMatching
                        info!(task_id = task.id, "自動マッチングタスクを実行します");
                        use crate::services::schedule::auto_matching_task_executor::AutoMatchingTaskExecutor;

                        let role_service = RoleNotificationService::new(
                            repos.all_recruitment_notification_roles,
                            repos.quest_recruitment_notification_roles,
                        );

                        let timezone_service = TimezoneService::new(repos.guild_settings);

                        let guild_env_service =
                            GuildEnvironmentService::new(repos.guild_environment);

                        let notification_management_service = NotificationManagementService::new(
                            repos.notification,
                            repos.notification_rel_battle_recruitment,
                            repos.scheduled_task,
                        );

                        let dismissal_service = DismissalManagementService::new(
                            repos.battle_recruitment_dismissal,
                            repos.scheduled_task,
                            repos.scheduled_task_dismissal,
                        );

                        let recruitment_creation_service =
                            Arc::new(RecruitmentCreationService::new(
                                repos.guild_channel,
                                repos.quest,
                                repos.battle_style,
                                role_service,
                                timezone_service,
                                guild_env_service,
                                repos.battle_recruitment_schedule_dismissal,
                                MessageService::new(repos.guild_message_text, repos.message_text),
                                notification_management_service,
                                dismissal_service,
                                repos.battle_recruitments,
                            ));

                        let matching_service = PeriodicMatchingService::new(
                            repos.auto_recruitment_participant,
                            repos.user_desired_quest,
                            repos.quest_matching,
                            repos.quest_matching_user,
                            repos.quest,
                        );

                        let executor = AutoMatchingTaskExecutor::new(
                            Arc::new(repos.scheduled_task),
                            recruitment_creation_service,
                            matching_service,
                            repos.auto_recruitment,
                            repos.quest_matching_user,
                            repos.quest_matching,
                            repos.quest,
                        );

                        match executor.execute(&txn, db, gateway.as_ref(), task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "自動マッチングタスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "自動マッチングタスクの実行中にエラーが発生しました");
                                if let Err(mark_err) =
                                    repos.scheduled_task.mark_as_failed(&txn, task.id).await
                                {
                                    error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                                }
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    _ => {
                        warn!(
                            task_id = task.id,
                            task_type = task.task_type,
                            "不明なタスクタイプです"
                        );
                        if let Err(mark_err) =
                            repos.scheduled_task.mark_as_failed(&txn, task.id).await
                        {
                            error!(task_id = task.id, error = %mark_err, "タスクの失敗マークに失敗しました");
                        }
                    }
                }
            } else {
                debug!(
                    task_id = task.id,
                    schedule_datetime = %task.schedule_datetime,
                    "タスクはまだ実行時刻に達していません"
                );
            }
        }

        // トランザクションをコミット
        txn.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
