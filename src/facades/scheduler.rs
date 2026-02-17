use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::db_helper::set_current_guild_id;
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

    /// ギルド向けにスケジュールを生成
    /// 指定したguildに関わるスケジュールのみ再生成する
    pub async fn generate_schedules_for_guild(
        &self,
        guild_id: i64,
        task_type: Option<ScheduledTaskType>,
    ) -> Result<()> {
        info!(
            guild_id,
            task_type = ?task_type,
            "ギルド向けスケジュール生成を開始します"
        );

        // ギルド向け再生成はGuildロールを使用
        let txn = self.app_state.guild_db().begin().await?;

        let repos = &self.app_state.repositories;
        let result = async {
            // RLSポリシー用セッション変数を設定
            set_current_guild_id(&txn, guild_id).await?;

            let service = SchedulerService::new(
                repos.schedule,
                repos.notification,
                repos.notification_rel_event_schedule,
                repos.scheduled_task,
                repos.battle_recruitment_schedule,
                repos.last_process_time,
            );
            service
                .generate_and_persist_schedules_for_guild(
                    &txn,
                    &self.app_state,
                    guild_id,
                    task_type,
                )
                .await?;
            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(
                    guild_id,
                    "ギルド向けスケジュール生成のトランザクションをコミットしました"
                );
                Ok(())
            }
            Err(e) => {
                error!(error = %e, guild_id, "ギルド向けスケジュール生成に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// 管理サーバー向けにスケジュールを生成
    /// 全guildのスケジュールを再生成する
    pub async fn generate_schedules_global(
        &self,
        task_type: Option<ScheduledTaskType>,
    ) -> Result<()> {
        info!(
            task_type = ?task_type,
            "管理サーバー向けスケジュール生成を開始します"
        );

        // 管理サーバー向け再生成はGlobalロールを使用
        let txn = self.app_state.global_db().begin().await?;

        let repos = &self.app_state.repositories;
        let result = async {
            let service = SchedulerService::new(
                repos.schedule,
                repos.notification,
                repos.notification_rel_event_schedule,
                repos.scheduled_task,
                repos.battle_recruitment_schedule,
                repos.last_process_time,
            );
            service
                .generate_and_persist_schedules_for_global(&txn, &self.app_state, task_type)
                .await?;
            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!("管理サーバー向けスケジュール生成のトランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "管理サーバー向けスケジュール生成に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// スケジュールを生成
    /// 既存呼び出しとの互換性のため、管理サーバー向け全体再生成に委譲
    pub async fn generate_schedules(&self) -> Result<()> {
        self.generate_schedules_global(None).await
    }
}
