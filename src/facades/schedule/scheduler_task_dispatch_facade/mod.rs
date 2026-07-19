use crate::di::Repositories;
use crate::gateway::DiscordGateway;
use crate::repository::{
    BattleRecruitmentsRepository, GuildMessageTextRepository, MessageTextRepository,
    RecruitmentParticipantsRepository,
};
use crate::services::message::MessageService;
use crate::services::schedule::SchedulerDispatchUseCase;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use tracing::{error, info};

mod auto_matching;
mod recurring_recruitment;
mod shared_presentation;
mod task_runners;
mod task_timing;

/// SchedulerTaskDispatchFacade向け募集リポジトリ境界
pub trait DispatchRecruitmentRepository: BattleRecruitmentsRepository + 'static {}

impl<T> DispatchRecruitmentRepository for T where T: BattleRecruitmentsRepository + 'static {}

/// SchedulerTaskDispatchFacade向け参加者リポジトリ境界
pub trait DispatchParticipantsRepository: RecruitmentParticipantsRepository + 'static {}

impl<T> DispatchParticipantsRepository for T where T: RecruitmentParticipantsRepository + 'static {}

/// SchedulerTaskDispatchFacade向けギルドメッセージリポジトリ境界
pub trait DispatchGuildMessageTextRepository:
    GuildMessageTextRepository + Clone + Send + Sync + 'static
{
}

impl<T> DispatchGuildMessageTextRepository for T where
    T: GuildMessageTextRepository + Clone + Send + Sync + 'static
{
}

/// SchedulerTaskDispatchFacade向けメッセージリポジトリ境界
pub trait DispatchMessageTextRepository:
    MessageTextRepository + Clone + Send + Sync + 'static
{
}

impl<T> DispatchMessageTextRepository for T where
    T: MessageTextRepository + Clone + Send + Sync + 'static
{
}

/// Scheduler向けタスク実行Facade
///
/// - トランザクション境界を管理する（唯一の責務）
/// - task_typeごとにExecutor/Serviceを組み立てて実行する（サービス合成）
///
/// 以前は組み立て・実行ロジックが `TaskDispatchService`（service層）にあったが、
/// 複数serviceを合成してユースケースを実行するのはfacade層の責務であるため、
/// このfacadeへ統合した。
pub struct SchedulerTaskDispatchFacade<R, P, GM, MT>
where
    R: DispatchRecruitmentRepository,
    P: DispatchParticipantsRepository,
    GM: DispatchGuildMessageTextRepository,
    MT: DispatchMessageTextRepository,
{
    db: Arc<DatabaseConnection>,
    recruitment_repo: Arc<R>,
    participants_repo: Arc<P>,
    message_service: Arc<MessageService<GM, MT>>,
    repos: Repositories,
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
        recruitment_repo: Arc<R>,
        participants_repo: Arc<P>,
        message_service: Arc<MessageService<GM, MT>>,
        repos: Repositories,
    ) -> Self {
        Self {
            db,
            recruitment_repo,
            participants_repo,
            message_service,
            repos,
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

        let result = self.run_repair(&txn, gateway).await;

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
            .run_dispatch_cycle(&txn, self.db.as_ref(), gateway)
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
