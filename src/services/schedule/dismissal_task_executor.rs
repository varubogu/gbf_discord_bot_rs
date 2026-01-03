use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::battle_recruitments_repository::SeaOrmBattleRecruitmentsRepository;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::recruitment_participants_repository::SeaOrmRecruitmentParticipantsRepository;
use crate::repository::database::schedule::{
    SeaOrmBattleRecruitmentDismissalRepository, SeaOrmScheduledTaskDismissalRepository,
    SeaOrmScheduledTaskRepository,
};
use crate::repository::quest_repository::QuestRepository;
use crate::repository::recruitment_participants_repository::RecruitmentParticipantsRepository;
use crate::repository::schedule::{
    BattleRecruitmentDismissalRepository, ScheduledTaskDismissalRepository, ScheduledTaskRepository,
};
use crate::services::message::{MessageService, MessageTextId};
use crate::services::schedule::NotificationManagementService;
use crate::types::{AppError, Result};
use poise::serenity_prelude::{ChannelId, EditMessage, Http, MessageId};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 解散タスク実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum DismissalExecutionResult {
    /// 実行成功（募集をキャンセルした）
    Dismissed { recruitment_id: i32 },
    /// 定員に達しているためスキップ
    SkippedDueToSufficientParticipants { recruitment_id: i32 },
    /// 募集が見つからない（既に削除済み）
    RecruitmentNotFound,
    /// 募集が既にキャンセル済み
    AlreadyCancelled { recruitment_id: i32 },
    /// Discord メッセージが見つからない
    DiscordMessageNotFound { recruitment_id: i32 },
}

/// 解散タスク実行サービス
pub struct DismissalTaskExecutor {
    message_service: Arc<MessageService>,
    guild_settings_repo: Arc<SeaOrmGuildSettingsRepository>,
}

impl DismissalTaskExecutor {
    pub fn new(message_service: Arc<MessageService>) -> Self {
        Self {
            message_service,
            guild_settings_repo: Arc::new(SeaOrmGuildSettingsRepository::new()),
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

    /// 解散タスクを実行する
    pub async fn execute(
        &self,
        txn: &DatabaseTransaction,
        http: &Arc<Http>,
        task_id: i32,
    ) -> Result<DismissalExecutionResult> {
        info!(task_id, "解散タスク実行開始");

        let task_repo = SeaOrmScheduledTaskRepository::new();
        let dismissal_repo = SeaOrmScheduledTaskDismissalRepository::new();
        let dismissal_setting_repo = SeaOrmBattleRecruitmentDismissalRepository::new();
        let recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
        let participants_repo = SeaOrmRecruitmentParticipantsRepository::new();
        let quest_repo = SeaOrmQuestRepository::new();

        // タスクが存在し、未実行であることを確認
        let task = match task_repo.find_by_id(txn, task_id).await? {
            Some(task) if !task.is_executed => task,
            Some(_) => {
                warn!(task_id, "タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("Task {task_id} is already executed"),
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
        let dismissal_rel = match dismissal_repo.find_by_task_id(txn, task_id).await? {
            Some(d) => d,
            None => {
                error!(task_id, "解散情報が見つかりません");
                return Err(AppError::Business {
                    message: format!("Dismissal info not found for task {task_id}"),
                });
            }
        };

        let recruitment_dismissal_id = dismissal_rel.recruitment_dismissal_id;

        // 解散設定から募集IDを取得
        let dismissal_setting = match dismissal_setting_repo
            .find_by_id(txn, recruitment_dismissal_id)
            .await?
        {
            Some(s) => s,
            None => {
                error!(
                    task_id,
                    recruitment_dismissal_id, "解散設定が見つかりません"
                );
                return Err(AppError::Business {
                    message: format!("Dismissal setting {recruitment_dismissal_id} not found"),
                });
            }
        };

        let recruitment_id = dismissal_setting.recruitment_id;

        // 募集情報を取得
        let recruitment = match recruitment_repo
            .get_by_id_with_txn(txn, recruitment_id)
            .await?
        {
            Some(r) => r,
            None => {
                warn!(
                    task_id,
                    recruitment_id, "募集が見つかりません（既に削除済み）"
                );
                task_repo.mark_as_executed(txn, task_id).await?;
                return Ok(DismissalExecutionResult::RecruitmentNotFound);
            }
        };

        // 既にキャンセル済みかチェック
        if recruitment.is_canceled {
            info!(task_id, recruitment_id, "募集は既にキャンセル済みです");
            task_repo.mark_as_executed(txn, task_id).await?;
            return Ok(DismissalExecutionResult::AlreadyCancelled { recruitment_id });
        }

        // 参加者数を取得
        let participant_count = participants_repo
            .count_unique_users(txn, recruitment_id)
            .await? as usize;

        // クエスト情報から定員を取得
        let quest = quest_repo
            .get_by_target_id(txn, recruitment.quest_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: format!("Quest {} not found", recruitment.quest_id),
            })?;

        let max_participants = quest.recruit_count as usize;

        info!(
            task_id,
            recruitment_id, participant_count, max_participants, "参加者数をチェックします"
        );

        // 定員に達している場合はスキップ
        if participant_count >= max_participants {
            info!(
                task_id,
                recruitment_id,
                participant_count,
                max_participants,
                "定員に達しているため解散をスキップします"
            );
            task_repo.mark_as_executed(txn, task_id).await?;
            return Ok(
                DismissalExecutionResult::SkippedDueToSufficientParticipants { recruitment_id },
            );
        }

        // 定員未達のため募集をキャンセル
        info!(
            task_id,
            recruitment_id, participant_count, max_participants, "定員未達のため募集を解散します"
        );

        // Discordメッセージを取得・編集
        let channel_id = ChannelId::new(recruitment.channel_id as u64);
        let message_id = MessageId::new(recruitment.message_id as u64);

        let original_message = match channel_id.message(http, message_id).await {
            Ok(msg) => msg,
            Err(e) => {
                warn!(
                    task_id,
                    recruitment_id,
                    channel_id = recruitment.channel_id,
                    message_id = recruitment.message_id,
                    error = %e,
                    "Discordメッセージが見つかりません"
                );
                task_repo.mark_as_executed(txn, task_id).await?;
                return Ok(DismissalExecutionResult::DiscordMessageNotFound { recruitment_id });
            }
        };

        let original_content = original_message.content.clone();

        // キャンセル済みメッセージを作成
        let guild_id_value = task
            .guild_id
            .unwrap_or_else(|| recruitment.guild_id.try_into().unwrap());
        let guild_id = Some(guild_id_value);
        let locale = self.get_guild_locale(txn, guild_id_value).await?;
        let cancelled_suffix = self
            .message_service
            .get_message(
                txn,
                MessageTextId::RecruitmentCommandCancelledMessageSuffix.as_str(),
                HashMap::new(),
                guild_id,
                Some(&locale),
            )
            .await
            .unwrap_or_else(|_| "この募集はキャンセルされました".to_string());

        let cancelled_content = format!("~~{original_content}~~\n\n**{cancelled_suffix}**");

        // メッセージを編集
        channel_id
            .edit_message(
                http,
                message_id,
                EditMessage::new().content(cancelled_content),
            )
            .await
            .map_err(|e| {
                error!(error = %e, "募集メッセージの編集に失敗しました");
                AppError::Discord(Box::new(e))
            })?;

        // 参加者リストを取得
        let participant_user_ids = participants_repo
            .get_all_participant_user_ids(txn, recruitment_id)
            .await?;

        // 解散通知メッセージを送信（元の募集メッセージへの返信として）
        let dismissal_notification = if participant_user_ids.is_empty() {
            // 参加者がいない場合
            self.message_service
                .get_message(
                    txn,
                    MessageTextId::RecruitmentNotificationDismissal.as_str(),
                    HashMap::new(),
                    guild_id,
                    Some(&locale),
                )
                .await
                .unwrap_or_else(|_| {
                    "人数が集まらなかったため、この募集は解散しました。".to_string()
                })
        } else {
            // 参加者がいる場合
            let base_message = self
                .message_service
                .get_message(
                    txn,
                    MessageTextId::RecruitmentNotificationDismissalWithParticipants.as_str(),
                    HashMap::new(),
                    guild_id,
                    Some(&locale),
                )
                .await
                .unwrap_or_else(|_| {
                    "人数が集まらなかったため、この募集は解散しました。\n参加予定だった皆さん"
                        .to_string()
                });

            // 参加者のメンションを作成
            let participants_str = participant_user_ids
                .iter()
                .map(|user_id| format!("<@{user_id}>"))
                .collect::<Vec<_>>()
                .join(" ");

            format!("{base_message}: {participants_str}")
        };

        channel_id
            .send_message(
                http,
                poise::serenity_prelude::CreateMessage::new()
                    .content(dismissal_notification)
                    .reference_message(&original_message),
            )
            .await
            .map_err(|e| {
                error!(error = %e, "解散通知メッセージの送信に失敗しました");
                AppError::Discord(Box::new(e))
            })?;

        // 募集をキャンセル状態に更新
        recruitment_repo
            .set_canceled_with_txn(txn, recruitment_id, message_id)
            .await?;

        // 募集に紐づく他の通知（出発5分前、出発時刻）を削除
        let notification_management_service = NotificationManagementService::new();
        let deleted_count = notification_management_service
            .delete_recruitment_notifications(txn, recruitment_id)
            .await?;

        info!(
            task_id,
            recruitment_id,
            deleted_notifications = deleted_count,
            "募集に紐づく通知を削除しました"
        );

        // タスクを実行済みにマーク
        task_repo.mark_as_executed(txn, task_id).await?;

        info!(
            task_id,
            recruitment_id, "解散タスク実行完了（募集をキャンセルしました）"
        );

        Ok(DismissalExecutionResult::Dismissed { recruitment_id })
    }
}
