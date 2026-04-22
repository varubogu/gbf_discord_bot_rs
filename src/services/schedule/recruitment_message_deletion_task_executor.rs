use crate::errors::GatewayError;
use crate::gateway::DiscordMessageGateway;
use crate::repository::BattleRecruitmentsRepository;
use crate::repository::schedule::{
    ScheduledTaskRecruitmentMessageDeletionRepository, ScheduledTaskRepository,
};
use crate::types::discord::{DiscordChannelId, DiscordMessageId};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{error, info, warn};

/// 募集投稿削除タスク実行結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecruitmentMessageDeletionExecutionResult {
    /// Discord投稿を削除した
    Deleted { recruitment_id: i32 },
    /// 関連行が見つからない
    RelationNotFound { task_id: i32 },
    /// 募集が見つからない
    RecruitmentNotFound { recruitment_id: i32 },
    /// DiscordメッセージIDが未設定
    MessageIdNotReady { recruitment_id: i32 },
    /// Discord上でメッセージが既に存在しない
    DiscordMessageNotFound { recruitment_id: i32 },
}

/// 募集投稿削除タスクExecutor
///
/// DB上の募集行は削除せず、元募集投稿のDiscordメッセージのみ削除する。
pub struct RecruitmentMessageDeletionTaskExecutor<ST, MD, R>
where
    ST: ScheduledTaskRepository,
    MD: ScheduledTaskRecruitmentMessageDeletionRepository,
    R: BattleRecruitmentsRepository,
{
    task_repo: Arc<ST>,
    message_deletion_repo: Arc<MD>,
    recruitment_repo: Arc<R>,
}

impl<ST, MD, R> RecruitmentMessageDeletionTaskExecutor<ST, MD, R>
where
    ST: ScheduledTaskRepository,
    MD: ScheduledTaskRecruitmentMessageDeletionRepository,
    R: BattleRecruitmentsRepository,
{
    pub fn new(
        task_repo: Arc<ST>,
        message_deletion_repo: Arc<MD>,
        recruitment_repo: Arc<R>,
    ) -> Self {
        Self {
            task_repo,
            message_deletion_repo,
            recruitment_repo,
        }
    }

    /// 募集投稿削除タスクを実行する
    pub async fn execute<G: DiscordMessageGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        task_id: i32,
    ) -> Result<RecruitmentMessageDeletionExecutionResult> {
        info!(task_id, "募集投稿削除タスク実行開始");

        match self.task_repo.find_by_id(txn, task_id).await? {
            Some(task) if task.execution_status.is_pending() => {}
            Some(_) => {
                warn!(task_id, "募集投稿削除タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("タスクは未実行状態ではありません: {task_id}"),
                });
            }
            None => {
                warn!(task_id, "募集投稿削除タスクが見つかりません");
                return Err(AppError::Business {
                    message: format!("タスクが見つかりません: {task_id}"),
                });
            }
        }

        let deletion = match self
            .message_deletion_repo
            .find_by_task_id(txn, task_id)
            .await?
        {
            Some(deletion) => deletion,
            None => {
                warn!(task_id, "募集投稿削除関連情報が見つかりません");
                self.task_repo
                    .mark_as_succeeded_with_warning(txn, task_id)
                    .await?;
                return Ok(RecruitmentMessageDeletionExecutionResult::RelationNotFound { task_id });
            }
        };

        let recruitment_id = deletion.recruitment_id;
        let recruitment = match self
            .recruitment_repo
            .get_by_id_with_txn(txn, recruitment_id)
            .await?
        {
            Some(recruitment) => recruitment,
            None => {
                warn!(task_id, recruitment_id, "削除対象の募集が見つかりません");
                self.task_repo
                    .mark_as_succeeded_with_warning(txn, task_id)
                    .await?;
                return Ok(
                    RecruitmentMessageDeletionExecutionResult::RecruitmentNotFound {
                        recruitment_id,
                    },
                );
            }
        };

        if recruitment.message_id == 0 {
            warn!(
                task_id,
                recruitment_id, "募集投稿のDiscordメッセージIDが未設定のため削除をスキップします"
            );
            self.task_repo
                .mark_as_succeeded_with_warning(txn, task_id)
                .await?;
            return Ok(
                RecruitmentMessageDeletionExecutionResult::MessageIdNotReady { recruitment_id },
            );
        }

        let channel_id = DiscordChannelId::new(recruitment.channel_id);
        let message_id = DiscordMessageId::new(recruitment.message_id);

        match gateway.delete_message(channel_id, message_id).await {
            Ok(()) => {
                self.task_repo.mark_as_succeeded(txn, task_id).await?;
                info!(
                    task_id,
                    recruitment_id,
                    channel_id = %channel_id,
                    message_id = %message_id,
                    "募集投稿のDiscordメッセージを削除しました"
                );
                Ok(RecruitmentMessageDeletionExecutionResult::Deleted { recruitment_id })
            }
            Err(error) if error.is_not_found() => {
                warn!(
                    task_id,
                    recruitment_id,
                    channel_id = %channel_id,
                    message_id = %message_id,
                    error = %error,
                    "募集投稿のDiscordメッセージが存在しないため警告付き成功にします"
                );
                self.task_repo
                    .mark_as_succeeded_with_warning(txn, task_id)
                    .await?;
                Ok(
                    RecruitmentMessageDeletionExecutionResult::DiscordMessageNotFound {
                        recruitment_id,
                    },
                )
            }
            Err(error) => {
                error!(
                    task_id,
                    recruitment_id,
                    channel_id = %channel_id,
                    message_id = %message_id,
                    error = %error,
                    "募集投稿のDiscordメッセージ削除に失敗しました"
                );
                self.mark_task_as_failed(txn, task_id).await;
                Err(map_gateway_error(error))
            }
        }
    }

    async fn mark_task_as_failed(&self, txn: &DatabaseTransaction, task_id: i32) {
        if let Err(mark_error) = self.task_repo.mark_as_failed(txn, task_id).await {
            error!(
                task_id,
                error = %mark_error,
                "募集投稿削除タスクの失敗ステータス更新に失敗しました"
            );
        }
    }
}

fn map_gateway_error(error: GatewayError) -> AppError {
    error.into()
}
