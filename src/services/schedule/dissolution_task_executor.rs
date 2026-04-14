use crate::gateway::DiscordGateway;
use crate::repository::GuildSettingsRepository;
use crate::repository::schedule::{ScheduledTaskDissolutionRepository, ScheduledTaskRepository};
use crate::repository::{
    BattleRecruitmentsRepository, GuildMessageTextRepository, MessageTextRepository,
    QuestRepository, RecruitmentParticipantsRepository,
};
use crate::services::message::MessageService;
use crate::services::recruitment::cancel::{
    create_cancel_notification_text, create_cancelled_message_content,
};
use crate::types::discord::{DiscordChannelId, DiscordMessageId, MessageContent};
use crate::types::{AppError, Result};
use crate::utils::datetime_display::format_datetime_with_weekday;
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 解散タスク実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum DissolutionExecutionResult {
    /// 実行成功（募集をキャンセルした）
    Cancelled { recruitment_id: i32 },
    /// 人数条件を満たしているためスキップ
    SkippedDueToSufficientParticipants { recruitment_id: i32 },
    /// 募集が見つからない（既に削除済み）
    RecruitmentNotFound { recruit_id: i32 },
    /// 募集が既にキャンセル済み
    AlreadyCancelled { recruitment_id: i32 },
    /// Discord メッセージが見つからない
    DiscordMessageNotFound { recruitment_id: i32 },
}

/// 解散タスク実行サービス
///
/// 設計: scheduled_task_dissolutions から募集情報を取得し、
/// 参加者数をチェックして、条件を満たさない場合は募集をキャンセルする
pub struct DissolutionTaskExecutor<R, P, ST, SD, GS, Q, GM, MT>
where
    R: BattleRecruitmentsRepository,
    P: RecruitmentParticipantsRepository,
    ST: ScheduledTaskRepository,
    SD: ScheduledTaskDissolutionRepository,
    GS: GuildSettingsRepository,
    Q: QuestRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
{
    task_repo: Arc<ST>,
    dissolution_repo: Arc<SD>,
    recruitment_repo: Arc<R>,
    participants_repo: Arc<P>,
    message_service: Arc<MessageService<GM, MT>>,
    guild_settings_repo: Arc<GS>,
    quest_repo: Q,
}

impl<R, P, ST, SD, GS, Q, GM, MT> DissolutionTaskExecutor<R, P, ST, SD, GS, Q, GM, MT>
where
    R: BattleRecruitmentsRepository,
    P: RecruitmentParticipantsRepository,
    ST: ScheduledTaskRepository,
    SD: ScheduledTaskDissolutionRepository,
    GS: GuildSettingsRepository,
    Q: QuestRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_repo: Arc<ST>,
        dissolution_repo: Arc<SD>,
        recruitment_repo: Arc<R>,
        participants_repo: Arc<P>,
        message_service: Arc<MessageService<GM, MT>>,
        guild_settings_repo: Arc<GS>,
        quest_repo: Q,
    ) -> Self {
        Self {
            task_repo,
            dissolution_repo,
            recruitment_repo,
            participants_repo,
            message_service,
            guild_settings_repo,
            quest_repo,
        }
    }

    /// ギルドのロケールを取得
    /// 未設定の場合はデフォルト（ja）を返す
    async fn get_guild_locale(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<String> {
        match self
            .guild_settings_repo
            .find_by_guild_id_with_txn(txn, guild_id)
            .await?
        {
            Some(settings) => Ok(settings.locale),
            None => {
                // 未設定の場合はデフォルト（ja）を返す
                debug!(
                    guild_id = guild_id,
                    "ロケール未設定のため、デフォルト（ja）を使用します"
                );
                Ok("ja".to_string())
            }
        }
    }

    /// 解散タスクを実行する（実行時DB再確認を含む）
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `gateway` - Discord Gateway
    /// * `task_id` - 実行対象のタスクID
    ///
    /// # 戻り値
    /// * `Ok(DissolutionExecutionResult)` - 実行結果
    ///
    /// # エラー
    /// * タスクが見つからない場合
    /// * DB操作でエラーが発生した場合
    pub async fn execute<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        task_id: i32,
    ) -> Result<DissolutionExecutionResult> {
        info!(task_id, "解散タスク実行開始");

        // 実行時DB再確認: タスクが削除されていないか、既に実行済みでないかを確認
        let _task = match self.task_repo.find_by_id(txn, task_id).await? {
            Some(task) if task.execution_status.is_pending() => task,
            Some(_) => {
                warn!(task_id, "タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("Task {task_id} is not pending"),
                });
            }
            None => {
                warn!(task_id, "タスクが見つかりません");
                return Err(AppError::Business {
                    message: format!("Task {task_id} not found"),
                });
            }
        };

        // 解散情報を取得
        let dissolution = match self.dissolution_repo.find_by_task_id(txn, task_id).await? {
            Some(d) => d,
            None => {
                error!(task_id, "解散情報が見つかりません");
                return Err(AppError::Business {
                    message: format!("Dissolution info not found for task {task_id}"),
                });
            }
        };

        let recruit_id = dissolution.recruit_id;

        // 募集情報を取得
        let recruitment = match self
            .recruitment_repo
            .get_by_id_with_txn(txn, recruit_id)
            .await?
        {
            Some(r) => r,
            None => {
                warn!(task_id, recruit_id, "募集が見つかりません（既に削除済み）");
                // 警告付きでタスクを完了マーク
                self.task_repo
                    .mark_as_succeeded_with_warning(txn, task_id)
                    .await?;
                return Ok(DissolutionExecutionResult::RecruitmentNotFound { recruit_id });
            }
        };

        // 既にキャンセル済みかチェック
        if recruitment.is_canceled {
            info!(
                task_id,
                recruit_id,
                recruitment_id = recruitment.id,
                "募集は既にキャンセル済みです"
            );
            // 警告付きでタスクを完了マーク
            self.task_repo
                .mark_as_succeeded_with_warning(txn, task_id)
                .await?;
            return Ok(DissolutionExecutionResult::AlreadyCancelled {
                recruitment_id: recruitment.id,
            });
        }

        // 解散タスクが実行される時刻に達したため、募集をキャンセルする
        info!(
            task_id,
            recruitment_id = recruitment.id,
            "解散タスク実行時刻に達したため募集をキャンセルします"
        );

        // Discordメッセージを取得
        let channel_id = DiscordChannelId::new(recruitment.channel_id as u64);
        let message_id = DiscordMessageId::new(recruitment.message_id as u64);

        let message = match gateway.get_message(channel_id, message_id).await {
            Ok(msg) => msg,
            Err(e) => {
                warn!(
                    task_id,
                    recruitment_id = recruitment.id,
                    channel_id = recruitment.channel_id,
                    message_id = recruitment.message_id,
                    error = %e,
                    "Discordメッセージが見つかりません"
                );
                // 警告付きでタスクを完了マーク
                self.task_repo
                    .mark_as_succeeded_with_warning(txn, task_id)
                    .await?;
                return Ok(DissolutionExecutionResult::DiscordMessageNotFound {
                    recruitment_id: recruitment.id,
                });
            }
        };

        // ロケール情報を取得
        let guild_id = Some(recruitment.guild_id as i64);
        let locale = self
            .get_guild_locale(txn, recruitment.guild_id as i64)
            .await?;

        // キャンセル済みメッセージを作成
        let cancelled_content = create_cancelled_message_content(
            txn,
            self.message_service.as_ref(),
            guild_id,
            Some(&locale),
            &message.content,
        )
        .await?;

        // Discordメッセージを更新（Gateway経由）
        let edit_content = MessageContent::text(&cancelled_content);
        gateway
            .edit_message(channel_id, message_id, edit_content)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    recruitment_id = recruitment.id,
                    "Discordメッセージの更新に失敗しました"
                );
                AppError::from(e)
            })?;

        // 募集をキャンセル済み状態に更新（cancel_message_id は 0 でキャンセルを表現）
        self.recruitment_repo
            .set_canceled_with_txn(txn, recruitment.id, DiscordMessageId::new(0))
            .await?;

        // 参加者のユーザーIDリストを取得
        let participant_user_ids = self
            .participants_repo
            .get_all_participant_user_ids_with_txn(txn, recruitment.id)
            .await?;

        let notification_text = create_cancel_notification_text(
            txn,
            self.message_service.as_ref(),
            guild_id,
            Some(&locale),
            &participant_user_ids,
        )
        .await?;

        // クエスト情報を取得してフォールバックコンテキストを作成
        let fallback_context = match self
            .quest_repo
            .get_by_target_id(txn, recruitment.quest_id)
            .await?
        {
            Some(quest) => {
                let quest_start_at_jst = recruitment
                    .quest_start_at
                    .with_timezone(&chrono_tz::Asia::Tokyo);
                format!(
                    "解散タスク通知 - {} / {}",
                    quest.name,
                    format_datetime_with_weekday(
                        quest_start_at_jst,
                        "%Y/%m/%d ({weekday}) %H:%M",
                        &locale,
                    )
                )
            }
            None => "解散タスク通知".to_string(),
        };

        // 通知メッセージを送信（Gateway経由、元のメッセージに返信、失敗時は文脈情報付きで送信）
        gateway
            .send_reply(
                channel_id,
                message_id,
                MessageContent::text(&notification_text),
                Some(fallback_context),
            )
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    recruitment_id = recruitment.id,
                    "通知メッセージの送信に失敗しました"
                );
                AppError::from(e)
            })?;

        // タスクを正常終了にマーク
        self.task_repo.mark_as_succeeded(txn, task_id).await?;

        info!(
            task_id,
            recruitment_id = recruitment.id,
            "解散タスク実行完了"
        );

        Ok(DissolutionExecutionResult::Cancelled {
            recruitment_id: recruitment.id,
        })
    }
}

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
