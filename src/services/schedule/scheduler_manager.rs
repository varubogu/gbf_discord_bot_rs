use crate::errors::ScheduleError;
use crate::gateway::DiscordGateway;
use crate::services::schedule::SchedulerDispatchUseCase;
use crate::types::{AppError, Result};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

/// スケジューラーマネージャー
///
/// cronジョブの起動と定期ディスパッチのトリガーに責務を限定する。
pub struct SchedulerManager<G, D>
where
    G: DiscordGateway + Send + Sync + 'static,
    D: SchedulerDispatchUseCase<G> + 'static,
{
    scheduler: JobScheduler,
    gateway: Arc<G>,
    dispatcher: Arc<D>,
}

impl<G, D> SchedulerManager<G, D>
where
    G: DiscordGateway + Send + Sync + 'static,
    D: SchedulerDispatchUseCase<G> + 'static,
{
    /// 新しいSchedulerManagerを作成
    pub async fn new(gateway: Arc<G>, dispatcher: Arc<D>) -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::from(ScheduleError::SchedulerInitialization(e.to_string())))?;

        Ok(Self {
            scheduler,
            gateway,
            dispatcher,
        })
    }

    /// スケジューラーを開始
    ///
    /// 10秒間隔でディスパッチを実行する。
    pub async fn start(&mut self) -> Result<()> {
        info!("SchedulerManagerを起動します");

        if let Err(e) = self.run_startup_repair().await {
            error!(
                error = %e,
                "起動時の自動募集日時チャンネル補正に失敗しました。スケジューラー起動は継続します"
            );
        }

        let gateway = Arc::clone(&self.gateway);
        let dispatcher = Arc::clone(&self.dispatcher);

        let job = Job::new_async("*/10 * * * * *", move |_uuid, _lock| {
            let gateway = Arc::clone(&gateway);
            let dispatcher = Arc::clone(&dispatcher);

            Box::pin(async move {
                if let Err(e) = dispatcher.dispatch_due_tasks(&gateway).await {
                    error!(error = %e, "タスクのプリロード・実行中にエラーが発生しました");
                }
            })
        })
        .map_err(|e| AppError::from(ScheduleError::JobCreation(e.to_string())))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::from(ScheduleError::JobRegistration(e.to_string())))?;
        self.scheduler
            .start()
            .await
            .map_err(|e| AppError::from(ScheduleError::SchedulerStart(e.to_string())))?;

        info!("SchedulerManagerを起動しました");
        Ok(())
    }

    /// スケジューラーを停止
    pub async fn stop(&mut self) -> Result<()> {
        info!("SchedulerManagerを停止します");
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AppError::from(ScheduleError::SchedulerShutdown(e.to_string())))?;
        info!("SchedulerManagerを停止しました");
        Ok(())
    }

    async fn run_startup_repair(&self) -> Result<()> {
        self.dispatcher.repair_on_startup(&self.gateway).await
    }

    #[cfg(test)]
    async fn run_dispatch_cycle(&self) -> Result<()> {
        self.dispatcher.dispatch_due_tasks(&self.gateway).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ScheduleError;
    use crate::gateway::r#impl::mock_discord_gateway::MockDiscordGateway;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct FakeDispatcher {
        startup_calls: AtomicUsize,
        dispatch_calls: AtomicUsize,
        fail_startup: AtomicBool,
        fail_dispatch: AtomicBool,
    }

    impl Default for FakeDispatcher {
        fn default() -> Self {
            Self {
                startup_calls: AtomicUsize::new(0),
                dispatch_calls: AtomicUsize::new(0),
                fail_startup: AtomicBool::new(false),
                fail_dispatch: AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl SchedulerDispatchUseCase<MockDiscordGateway> for FakeDispatcher {
        async fn repair_on_startup(&self, _gateway: &Arc<MockDiscordGateway>) -> Result<()> {
            self.startup_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_startup.load(Ordering::SeqCst) {
                return Err(AppError::from(ScheduleError::StartupRepairFailed));
            }
            Ok(())
        }

        async fn dispatch_due_tasks(&self, _gateway: &Arc<MockDiscordGateway>) -> Result<()> {
            self.dispatch_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_dispatch.load(Ordering::SeqCst) {
                return Err(AppError::from(ScheduleError::DispatchFailed));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn run_startup_repair_delegates_to_dispatcher() {
        let gateway = Arc::new(MockDiscordGateway::new());
        let dispatcher = Arc::new(FakeDispatcher::default());

        let manager = SchedulerManager::new(gateway, Arc::clone(&dispatcher))
            .await
            .expect("manager should be created");

        manager
            .run_startup_repair()
            .await
            .expect("startup repair should succeed");

        assert_eq!(dispatcher.startup_calls.load(Ordering::SeqCst), 1);
        assert_eq!(dispatcher.dispatch_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn run_dispatch_cycle_returns_dispatch_error() {
        let gateway = Arc::new(MockDiscordGateway::new());
        let dispatcher = Arc::new(FakeDispatcher::default());
        dispatcher.fail_dispatch.store(true, Ordering::SeqCst);

        let manager = SchedulerManager::new(gateway, Arc::clone(&dispatcher))
            .await
            .expect("manager should be created");

        let err = manager
            .run_dispatch_cycle()
            .await
            .expect_err("dispatch should fail");

        match err {
            AppError::Business { message } => {
                assert_eq!(message, "スケジュールディスパッチに失敗しました");
            }
            other => panic!("unexpected error: {other}"),
        }

        assert_eq!(dispatcher.startup_calls.load(Ordering::SeqCst), 0);
        assert_eq!(dispatcher.dispatch_calls.load(Ordering::SeqCst), 1);
    }
}
