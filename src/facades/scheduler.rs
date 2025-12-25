use crate::services::schedule::scheduler_service::SchedulerService;
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// スケジューラーFacade
/// スケジュール管理の協調とトランザクション管理を担当
pub struct SchedulerFacade {
    app_state: Arc<AppState>,
}

impl SchedulerFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// スケジュールを生成
    /// イベントスケジュールと詳細から通知スケジュールを計算してDBに保存
    pub async fn generate_schedules(&self) -> Result<()> {
        info!("スケジュール生成を開始します");

        // スケジュール生成はSystemロールを使用（全ギルド対象）
        let txn = self.app_state.system_db().begin().await?;

        let result = async {
            let service = SchedulerService::new();
            service
                .generate_and_persist_schedules(&txn, &self.app_state)
                .await?;
            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!("スケジュール生成のトランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "スケジュール生成に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}
