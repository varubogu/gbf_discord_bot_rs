use crate::repository::database::schedule::{
    SeaOrmBattleRecruitmentScheduleRepository, SeaOrmScheduledTaskDissolutionRepository,
    SeaOrmScheduledTaskRecurringRecruitmentRepository, SeaOrmScheduledTaskRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::repository::{BattleRecruitmentsRepository, RecruitmentParticipantsRepository};
use crate::services::message::MessageService;
use crate::services::recruitment::recruitment_creation_service::RecruitmentCreationService;
use crate::services::schedule::dismissal_task_executor::DismissalTaskExecutor;
use crate::services::schedule::dissolution_task_executor::DissolutionTaskExecutor;
use crate::services::schedule::recurring_recruitment_task_executor::RecurringRecruitmentTaskExecutor;
use crate::services::schedule::{NotificationService, RecruitmentScheduleService};
use crate::types::Result;
use chrono::{Duration, Utc};
use poise::serenity_prelude::Http;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, warn};

/// スケジューラーマネージャー
///
/// tokio-cron-schedulerを使用して、定期的にタスクをプリロードし、実行する
pub struct SchedulerManager<
    R: BattleRecruitmentsRepository + 'static,
    P: RecruitmentParticipantsRepository + 'static,
> {
    scheduler: JobScheduler,
    db: Arc<DatabaseConnection>,
    http: Arc<Http>,
    task_repo: Arc<SeaOrmScheduledTaskRepository>,
    dissolution_repo: Arc<SeaOrmScheduledTaskDissolutionRepository>,
    recruitment_repo: Arc<R>,
    participants_repo: Arc<P>,
    message_service: Arc<MessageService>,
}

impl<R: BattleRecruitmentsRepository + 'static, P: RecruitmentParticipantsRepository + 'static>
    SchedulerManager<R, P>
{
    /// 新しいSchedulerManagerを作成
    pub async fn new(
        db: Arc<DatabaseConnection>,
        http: Arc<Http>,
        task_repo: Arc<SeaOrmScheduledTaskRepository>,
        dissolution_repo: Arc<SeaOrmScheduledTaskDissolutionRepository>,
        recruitment_repo: Arc<R>,
        participants_repo: Arc<P>,
        message_service: Arc<MessageService>,
    ) -> Result<Self> {
        let scheduler = JobScheduler::new().await.map_err(|e| {
            crate::types::AppError::Generic(format!("JobScheduler creation failed: {e}"))
        })?;

        Ok(Self {
            scheduler,
            db,
            http,
            task_repo,
            dissolution_repo,
            recruitment_repo,
            participants_repo,
            message_service,
        })
    }

    /// スケジューラーを開始
    ///
    /// 10秒間隔でタスクをプリロードするジョブを登録し、スケジューラーを起動する
    pub async fn start(&mut self) -> Result<()> {
        info!("SchedulerManagerを起動します");

        // 10秒間隔でタスクをプリロードするジョブを作成
        let db = Arc::clone(&self.db);
        let http = Arc::clone(&self.http);
        let task_repo = Arc::clone(&self.task_repo);
        let dissolution_repo = Arc::clone(&self.dissolution_repo);
        let recruitment_repo = Arc::clone(&self.recruitment_repo);
        let participants_repo = Arc::clone(&self.participants_repo);
        let message_service = Arc::clone(&self.message_service);

        let job = Job::new_async("*/10 * * * * *", move |_uuid, _lock| {
            let db = Arc::clone(&db);
            let http = Arc::clone(&http);
            let task_repo = Arc::clone(&task_repo);
            let dissolution_repo = Arc::clone(&dissolution_repo);
            let recruitment_repo = Arc::clone(&recruitment_repo);
            let participants_repo = Arc::clone(&participants_repo);
            let message_service = Arc::clone(&message_service);

            Box::pin(async move {
                if let Err(e) = Self::preload_and_execute_tasks(
                    &db,
                    &http,
                    &task_repo,
                    &dissolution_repo,
                    &recruitment_repo,
                    &participants_repo,
                    &message_service,
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

    /// タスクをプリロードして実行
    ///
    /// 現在時刻から20秒先までの未実行タスクを取得し、実行時刻に達しているものを実行する
    async fn preload_and_execute_tasks(
        db: &Arc<DatabaseConnection>,
        http: &Arc<Http>,
        task_repo: &Arc<SeaOrmScheduledTaskRepository>,
        dissolution_repo: &Arc<SeaOrmScheduledTaskDissolutionRepository>,
        recruitment_repo: &Arc<R>,
        participants_repo: &Arc<P>,
        message_service: &Arc<MessageService>,
    ) -> Result<()> {
        let now = Utc::now();
        let preload_until = now + Duration::seconds(20);

        debug!(
            to = %preload_until,
            "タスクをプリロードします"
        );

        // トランザクション開始
        let txn = db.begin().await?;

        // 未実行タスクを取得（scheduled_tasks）
        // is_executed=falseで絞り込んでいるため、過去の未実行タスクも取得される
        let tasks = task_repo.find_pending_to(&txn, preload_until).await?;

        if tasks.is_empty() {
            debug!("プリロード対象のタスクはありません");
            txn.commit().await?;
            return Ok(());
        }

        info!(tasks = tasks.len(), "タスクをプリロードしました");

        // scheduled_tasksを実行

        use crate::repository::database::schedule::SeaOrmNotificationRepository;
        use crate::repository::schedule::NotificationRepository as NotificationRepositoryTrait;

        let notification_service = NotificationService::new(Arc::clone(http));
        let notification_repo = SeaOrmNotificationRepository::new();

        for task in tasks {
            if task.schedule_datetime <= now {
                // 実行時刻に達している場合、タスクを実行
                match task.task_type {
                    1 => {
                        // Notification
                        info!(task_id = task.id, "通知タスクを実行します");

                        // task_idからnotificationsテーブルを検索
                        match notification_repo.find_by_task_id(&txn, task.id).await {
                            Ok(Some(notification)) => {
                                // 通知を送信
                                match notification_service
                                    .send_single_notification(&txn, &notification)
                                    .await
                                {
                                    Ok(_) => {
                                        // タスクを完了としてマーク
                                        if let Err(e) =
                                            task_repo.mark_as_executed(&txn, task.id).await
                                        {
                                            error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                                        }
                                        info!(task_id = task.id, "通知タスクを実行しました");
                                    }
                                    Err(e) => {
                                        error!(task_id = task.id, error = %e, "通知の送信中にエラーが発生しました");
                                    }
                                }
                            }
                            Ok(None) => {
                                // データ不整合：notificationsテーブルに通知がない
                                // このタスクは実行不可能なため、実行済みとしてマークして次回以降スキップ
                                warn!(
                                    task_id = task.id,
                                    "通知が見つかりません（データ不整合）。タスクを実行済みとしてマークします"
                                );
                                if let Err(e) = task_repo.mark_as_executed(&txn, task.id).await {
                                    error!(task_id = task.id, error = %e, "タスクの完了マークに失敗しました");
                                }
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "通知の取得に失敗しました");
                            }
                        }
                    }
                    2 => {
                        // Dissolution
                        info!(task_id = task.id, "解散タスクを実行します");
                        let executor = DissolutionTaskExecutor::new(
                            Arc::clone(task_repo),
                            Arc::clone(dissolution_repo),
                            Arc::clone(recruitment_repo),
                            Arc::clone(participants_repo),
                            Arc::clone(message_service),
                        );

                        match executor.execute(&txn, http, task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "解散タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "解散タスクの実行中にエラーが発生しました");
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    3 => {
                        // DataCleanup
                        warn!(task_id = task.id, "DataCleanupタスクは未実装です");
                        // TODO: CleanupTaskExecutorを実装
                    }
                    4 => {
                        // RecurringRecruitment
                        info!(task_id = task.id, "定期募集タスクを実行します");
                        let recurring_repo =
                            Arc::new(SeaOrmScheduledTaskRecurringRecruitmentRepository::new());
                        let schedule_repo =
                            Arc::new(SeaOrmBattleRecruitmentScheduleRepository::new());
                        let schedule_service = Arc::new(RecruitmentScheduleService::new());
                        let recruitment_creation_service =
                            Arc::new(RecruitmentCreationService::new());

                        let executor = RecurringRecruitmentTaskExecutor::new(
                            Arc::clone(task_repo),
                            recurring_repo,
                            schedule_repo,
                            schedule_service,
                            recruitment_creation_service,
                        );

                        match executor.execute(&txn, db, http, task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "定期募集タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "定期募集タスクの実行中にエラーが発生しました");
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    5 => {
                        // Dismissal
                        info!(task_id = task.id, "解散（人数不足）タスクを実行します");
                        let executor = DismissalTaskExecutor::new(Arc::clone(message_service));

                        match executor.execute(&txn, http, task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "解散（人数不足）タスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "解散（人数不足）タスクの実行中にエラーが発生しました");
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
                        use crate::repository::database::auto_recruitment::SeaOrmAutoRecruitmentChannelRepository;
                        use crate::services::schedule::auto_recruitment_rotation_task_executor::AutoRecruitmentRotationTaskExecutor;

                        let channel_repo = Arc::new(SeaOrmAutoRecruitmentChannelRepository::new());
                        let executor = AutoRecruitmentRotationTaskExecutor::new(
                            Arc::clone(task_repo),
                            channel_repo,
                        );

                        match executor.execute(&txn, http, task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "自動募集日付ローテーションタスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "自動募集日付ローテーションタスクの実行中にエラーが発生しました");
                                // エラーがあっても他のタスクは継続
                            }
                        }
                    }
                    7 => {
                        // AutoMatching
                        info!(task_id = task.id, "自動マッチングタスクを実行します");
                        use crate::services::schedule::auto_matching_task_executor::AutoMatchingTaskExecutor;

                        let recruitment_creation_service =
                            Arc::new(RecruitmentCreationService::new());
                        let executor = AutoMatchingTaskExecutor::new(
                            Arc::clone(task_repo),
                            recruitment_creation_service,
                        );

                        match executor.execute(&txn, db, http, task.id).await {
                            Ok(result) => {
                                info!(task_id = task.id, result = ?result, "自動マッチングタスクを実行しました");
                            }
                            Err(e) => {
                                error!(task_id = task.id, error = %e, "自動マッチングタスクの実行中にエラーが発生しました");
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
