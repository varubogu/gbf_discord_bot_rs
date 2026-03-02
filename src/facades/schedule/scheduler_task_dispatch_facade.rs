use crate::gateway::DiscordGateway;
use crate::services::schedule::task_dispatch_service::{
    DispatchGuildMessageTextRepository, DispatchMessageTextRepository,
    DispatchParticipantsRepository, DispatchRecruitmentRepository,
};
use crate::services::schedule::{SchedulerDispatchUseCase, TaskDispatchService};
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use tracing::{error, info};

/// Scheduler向けタスク実行Facade
///
/// トランザクション境界を管理し、TaskDispatchServiceへ処理を委譲する。
pub struct SchedulerTaskDispatchFacade<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    db: Arc<DatabaseConnection>,
    task_dispatch_service: TaskDispatchService<R, P, GM, MT>,
}

impl<R, P, GM, MT> SchedulerTaskDispatchFacade<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    pub fn new(
        db: Arc<DatabaseConnection>,
        task_dispatch_service: TaskDispatchService<R, P, GM, MT>,
    ) -> Self {
        Self {
            db,
            task_dispatch_service,
        }
    }
}

#[async_trait]
impl<G, R, P, GM, MT> SchedulerDispatchUseCase<G> for SchedulerTaskDispatchFacade<R, P, GM, MT>
where
    G: DiscordGateway + Send + Sync + 'static,
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    async fn repair_on_startup(&self, gateway: &Arc<G>) -> Result<()> {
        let txn = self.db.begin().await?;

        let result = self
            .task_dispatch_service
            .repair_auto_recruitment_channels_on_startup(&txn, gateway)
            .await;

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

    async fn dispatch_due_tasks(&self, gateway: &Arc<G>) -> Result<()> {
        let txn = self.db.begin().await?;

        let result = self
            .task_dispatch_service
            .dispatch_due_tasks(&txn, self.db.as_ref(), gateway)
            .await;

        match result {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "スケジューラーのタスク実行に失敗しました");
                if let Err(rollback_err) = txn.rollback().await {
                    error!(
                        error = %rollback_err,
                        "スケジューラー実行トランザクションのロールバックに失敗しました"
                    );
                }
                Err(e)
            }
        }
    }
}
