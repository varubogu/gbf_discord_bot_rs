//! 自動マッチングタスク実行サービス
//!
//! 10秒間隔で実行され、同じクエスト・時間・属性を希望するユーザーをマッチングし、
//! マッチング通知を送信後、マルチ募集を作成する

use crate::gateway::DiscordGateway;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::presenter::NotificationPresenter;
use crate::repository::auto_recruitment::{
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::recruitment::recruitment_creation_service::{
    CreatedMatchingRecruitmentInfo, MatchingRecruitmentParams, RecruitmentCreationService,
};
use crate::types::discord::{DiscordChannelId, DiscordMessageId};
use crate::types::{AppError, Result};
use chrono::{Datelike, Duration, TimeZone, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

type SharedRecruitmentCreationService<
    GC,
    Q,
    BS,
    A,
    QR,
    GE,
    SD,
    GM,
    MT,
    NMN,
    NMR,
    NMS,
    DR,
    TR,
    TDR,
    GS,
    BR,
    MDS,
> = Arc<
    RecruitmentCreationService<
        GC,
        Q,
        BS,
        A,
        QR,
        GE,
        SD,
        GM,
        MT,
        NMN,
        NMR,
        NMS,
        DR,
        TR,
        TDR,
        GS,
        BR,
        MDS,
    >,
>;

/// マッチング実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum AutoMatchingResult {
    /// 実行成功
    Success {
        /// マッチしたグループ数
        matched_groups: usize,
        /// 次回タスクID
        next_task_id: i32,
    },
    /// マッチング対象なし
    NoMatches { next_task_id: i32 },
}

/// 自動マッチングタスク実行サービス
pub struct AutoMatchingTaskExecutor<
    GC,
    Q,
    BS,
    A,
    QR,
    GE,
    SD,
    GM,
    MT,
    NMN,
    NMR,
    NMS,
    DR,
    TR,
    TDR,
    GS,
    BR,
    MDS,
    ST,
    ARR,
    QMR,
    QMUR,
    APR,
    UDR,
    RMR,
    RMQ,
> where
    GC: crate::repository::GuildChannelRepository,
    Q: crate::repository::QuestRepository,
    BS: crate::repository::BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: crate::repository::BattleRecruitmentsRepository,
    MDS: crate::services::schedule::RecruitmentMessageDeletionScheduler,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    APR: crate::repository::auto_recruitment::AutoRecruitmentParticipantRepository,
    UDR: crate::repository::auto_recruitment::UserDesiredQuestRepository,
    RMR: crate::repository::auto_recruitment::AutoRecruitmentMatchRuleRepository,
    RMQ: crate::repository::auto_recruitment::AutoRecruitmentMatchRuleQuotaRepository,
{
    task_repo: Arc<ST>,
    #[allow(clippy::type_complexity)]
    recruitment_creation_service: SharedRecruitmentCreationService<
        GC,
        Q,
        BS,
        A,
        QR,
        GE,
        SD,
        GM,
        MT,
        NMN,
        NMR,
        NMS,
        DR,
        TR,
        TDR,
        GS,
        BR,
        MDS,
    >,
    matching_service: PeriodicMatchingService<APR, UDR, QMR, QMUR, Q, RMR, RMQ>,
    auto_recruitment_repo: ARR,
    matching_user_repo: QMUR,
    matching_repo: QMR,
    quest_repo: Q,
}

impl<
    GC,
    Q,
    BS,
    A,
    QR,
    GE,
    SD,
    GM,
    MT,
    NMN,
    NMR,
    NMS,
    DR,
    TR,
    TDR,
    GS,
    BR,
    MDS,
    ST,
    ARR,
    QMR,
    QMUR,
    APR,
    UDR,
    RMR,
    RMQ,
>
    AutoMatchingTaskExecutor<
        GC,
        Q,
        BS,
        A,
        QR,
        GE,
        SD,
        GM,
        MT,
        NMN,
        NMR,
        NMS,
        DR,
        TR,
        TDR,
        GS,
        BR,
        MDS,
        ST,
        ARR,
        QMR,
        QMUR,
        APR,
        UDR,
        RMR,
        RMQ,
    >
where
    GC: crate::repository::GuildChannelRepository,
    Q: crate::repository::QuestRepository + Clone,
    BS: crate::repository::BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: crate::repository::BattleRecruitmentsRepository,
    MDS: crate::services::schedule::RecruitmentMessageDeletionScheduler,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    APR: crate::repository::auto_recruitment::AutoRecruitmentParticipantRepository,
    UDR: crate::repository::auto_recruitment::UserDesiredQuestRepository,
    RMR: crate::repository::auto_recruitment::AutoRecruitmentMatchRuleRepository,
    RMQ: crate::repository::auto_recruitment::AutoRecruitmentMatchRuleQuotaRepository,
{
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn new(
        task_repo: Arc<ST>,
        recruitment_creation_service: SharedRecruitmentCreationService<
            GC,
            Q,
            BS,
            A,
            QR,
            GE,
            SD,
            GM,
            MT,
            NMN,
            NMR,
            NMS,
            DR,
            TR,
            TDR,
            GS,
            BR,
            MDS,
        >,
        matching_service: PeriodicMatchingService<APR, UDR, QMR, QMUR, Q, RMR, RMQ>,
        auto_recruitment_repo: ARR,
        matching_user_repo: QMUR,
        matching_repo: QMR,
        quest_repo: Q,
    ) -> Self {
        Self {
            task_repo,
            recruitment_creation_service,
            matching_service,
            auto_recruitment_repo,
            matching_user_repo,
            matching_repo,
            quest_repo,
        }
    }

    /// 自動マッチングタスクを実行する
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `db_conn` - データベース接続
    /// * `gateway` - Discord Gateway
    /// * `task_id` - 実行対象のタスクID
    ///
    /// # 戻り値
    /// * `Ok(AutoMatchingResult)` - 実行結果
    pub async fn execute<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &G,
        task_id: i32,
    ) -> Result<AutoMatchingResult> {
        info!(task_id, "自動マッチングタスク実行開始");

        // タスクが削除されていないか、既に実行済みでないかを確認
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

        // マッチング処理を実行
        let matchings = self.matching_service.process_matching(txn).await?;

        let matched_groups = matchings.len();

        // マッチング通知を送信し、マルチ募集を作成
        if !matchings.is_empty() {
            self.send_match_notifications_and_create_recruitments(
                txn, db_conn, gateway, &matchings,
            )
            .await?;
        }

        // タスクを正常終了にマーク
        self.task_repo.mark_as_succeeded(txn, task_id).await?;

        // 次回タスクを作成（10秒後）
        let next_task_id = self.create_next_scheduled_task(txn).await?;

        if matched_groups > 0 {
            info!(
                task_id,
                matched_groups, next_task_id, "自動マッチングタスク実行完了"
            );
            Ok(AutoMatchingResult::Success {
                matched_groups,
                next_task_id,
            })
        } else {
            debug!(task_id, next_task_id, "マッチング対象なし");
            Ok(AutoMatchingResult::NoMatches { next_task_id })
        }
    }

    /// マッチング通知を送信し、マルチ募集を作成
    async fn send_match_notifications_and_create_recruitments<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &G,
        matchings: &[crate::models::entities::worker::quest_matchings::Model],
    ) -> Result<()> {
        // ギルドごとにグルーピング
        let mut guild_matchings: HashMap<i64, Vec<_>> = HashMap::new();
        for matching in matchings {
            guild_matchings
                .entry(matching.guild_id)
                .or_default()
                .push(matching);
        }

        for (guild_id, guild_matches) in guild_matchings {
            // 自動募集設定を取得
            let auto_recruitment = match self
                .auto_recruitment_repo
                .find_by_guild_id(txn, guild_id)
                .await?
            {
                Some(ar) => ar,
                None => {
                    warn!(guild_id, "自動募集設定が見つかりません");
                    continue;
                }
            };

            let matching_channel_id = match auto_recruitment.matching_channel_id {
                Some(id) => id as u64,
                None => {
                    warn!(guild_id, "マッチングチャンネルが設定されていません");
                    continue;
                }
            };

            for matching in guild_matches {
                // クエスト情報を取得
                let quest = match self
                    .quest_repo
                    .get_by_target_id(txn, matching.quest_id)
                    .await?
                {
                    Some(q) => q,
                    None => {
                        warn!(
                            guild_id,
                            quest_id = matching.quest_id,
                            "クエストが見つかりません"
                        );
                        continue;
                    }
                };

                // 参加ユーザーを取得
                let users = self
                    .matching_user_repo
                    .find_active_by_matching(txn, guild_id, matching.id)
                    .await?;

                if users.is_empty() {
                    continue;
                }

                let user_ids: Vec<u64> = users.iter().map(|u| u.user_id as u64).collect();

                // 通知を送信（Gateway経由）
                let notification_message_id = match self
                    .send_notification(
                        gateway,
                        matching_channel_id,
                        &quest.name,
                        matching.scheduled_month,
                        matching.scheduled_day,
                        matching.scheduled_hour,
                        &users
                            .iter()
                            .map(|u| (u.user_id as u64, u.battle_style_id))
                            .collect::<Vec<_>>(),
                    )
                    .await
                {
                    Ok(message_id) => Some(message_id),
                    Err(e) => {
                        error!(
                            error = %e,
                            guild_id,
                            matching_id = %matching.id,
                            "マッチング通知の送信に失敗しました"
                        );
                        // 通知失敗しても募集作成は試みる
                        None
                    }
                };

                // 出発時刻を計算
                let quest_start_at = self.calculate_quest_start_at(
                    matching.scheduled_month,
                    matching.scheduled_day,
                    matching.scheduled_hour,
                );

                // 出発時刻が過去の場合はスキップ
                let now = Utc::now();
                if quest_start_at <= now {
                    info!(
                        guild_id,
                        matching_id = %matching.id,
                        quest_start_at = %quest_start_at,
                        now = %now,
                        "出発時刻が過去のためマルチ募集の作成をスキップしました"
                    );
                    continue;
                }

                // マルチ募集を作成
                let params = MatchingRecruitmentParams {
                    guild_id,
                    quest_id: matching.quest_id,
                    quest_start_at,
                    participant_user_ids: user_ids,
                };

                match self
                    .recruitment_creation_service
                    .create_recruitment_from_matching(txn, db_conn, gateway, &params)
                    .await
                {
                    Ok(recruitment) => {
                        info!(
                            guild_id,
                            matching_id = %matching.id,
                            recruitment_id = recruitment.recruitment_id,
                            "マルチ募集を作成しました"
                        );

                        // マッチングに募集IDを設定
                        if let Err(e) = self
                            .matching_repo
                            .set_recruitment_id(
                                txn,
                                guild_id,
                                matching.id,
                                recruitment.recruitment_id,
                            )
                            .await
                        {
                            error!(
                                error = %e,
                                guild_id,
                                matching_id = %matching.id,
                                recruitment_id = recruitment.recruitment_id,
                                "マッチングへの募集ID設定に失敗しました"
                            );
                        }

                        if let Some(notification_message_id) = notification_message_id
                            && let Err(e) = self
                                .edit_notification_with_recruitment_link(
                                    gateway,
                                    matching_channel_id,
                                    notification_message_id,
                                    guild_id,
                                    &quest.name,
                                    matching.scheduled_month,
                                    matching.scheduled_day,
                                    matching.scheduled_hour,
                                    &users
                                        .iter()
                                        .map(|u| (u.user_id as u64, u.battle_style_id))
                                        .collect::<Vec<_>>(),
                                    &recruitment,
                                )
                                .await
                        {
                            error!(
                                error = %e,
                                guild_id,
                                matching_id = %matching.id,
                                recruitment_id = recruitment.recruitment_id,
                                "マッチング通知への募集リンク追記に失敗しました"
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            guild_id,
                            matching_id = %matching.id,
                            "マルチ募集の作成に失敗しました"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// スケジュール情報から出発時刻を計算
    fn calculate_quest_start_at(&self, month: i32, day: i32, hour: i32) -> chrono::DateTime<Utc> {
        // 現在の年を使用
        let now = Utc::now();
        let year = now.year();

        // hourが24以上の場合は翌日扱い（グラブルの5:00-28:00表記対応）
        let (actual_day, actual_hour) = if hour >= 24 {
            (day + 1, hour - 24)
        } else {
            (day, hour)
        };

        // 日本時間で構築してUTCに変換
        let jst = chrono_tz::Asia::Tokyo;
        let local_datetime = jst
            .with_ymd_and_hms(
                year,
                month as u32,
                actual_day as u32,
                actual_hour as u32,
                0,
                0,
            )
            .single()
            .unwrap_or_else(|| {
                // 年をまたぐ場合は翌年を試す
                jst.with_ymd_and_hms(
                    year + 1,
                    month as u32,
                    actual_day as u32,
                    actual_hour as u32,
                    0,
                    0,
                )
                .single()
                .expect("日時の構築に失敗しました")
            });

        local_datetime.with_timezone(&Utc)
    }

    /// 個別のマッチング通知を送信（Gateway経由）
    #[allow(clippy::too_many_arguments)]
    async fn send_notification<G: DiscordGateway>(
        &self,
        gateway: &G,
        channel_id: u64,
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
        users: &[(u64, Option<i32>)],
    ) -> Result<DiscordMessageId> {
        let channel = DiscordChannelId::new(channel_id);
        let message_content = NotificationPresenter::create_auto_matching_notification(
            quest_name, month, day, hour, users, None,
        );

        let message_id = gateway
            .send_message(channel, message_content)
            .await
            .map_err(|e| AppError::Business {
                message: format!("マッチング通知の送信に失敗しました: {e}"),
            })?;

        info!(
            channel_id,
            quest_name,
            month,
            day,
            hour,
            user_count = users.len(),
            "マッチング通知を送信しました"
        );

        Ok(message_id)
    }

    /// 募集作成後にマッチング通知へジャンプリンクを追記
    #[allow(clippy::too_many_arguments)]
    async fn edit_notification_with_recruitment_link<G: DiscordGateway>(
        &self,
        gateway: &G,
        notification_channel_id: u64,
        notification_message_id: DiscordMessageId,
        guild_id: i64,
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
        users: &[(u64, Option<i32>)],
        recruitment: &CreatedMatchingRecruitmentInfo,
    ) -> Result<()> {
        let channel = DiscordChannelId::new(notification_channel_id);
        let recruitment_url = format!(
            "https://discord.com/channels/{guild_id}/{}/{}",
            recruitment.channel_id, recruitment.message_id
        );
        let message_content = NotificationPresenter::create_auto_matching_notification(
            quest_name,
            month,
            day,
            hour,
            users,
            Some(&recruitment_url),
        );

        gateway
            .edit_message(channel, notification_message_id, message_content)
            .await
            .map_err(|e| AppError::Business {
                message: format!("マッチング通知の編集に失敗しました: {e}"),
            })?;

        Ok(())
    }

    /// 次回実行タスクを作成（10秒後）
    async fn create_next_scheduled_task(&self, txn: &DatabaseTransaction) -> Result<i32> {
        let next_execution = Utc::now() + Duration::seconds(10);

        debug!(
            next_execution = %next_execution,
            "次回自動マッチングタスクを作成します"
        );

        let task = self
            .task_repo
            .create(
                txn,
                next_execution,
                ScheduledTaskType::AutoMatching as i32,
                None, // guild_id: 全ギルド対象
                None, // channel_id
            )
            .await?;

        debug!(
            task_id = task.id,
            next_execution = %next_execution,
            "次回自動マッチングタスクを作成しました"
        );

        Ok(task.id)
    }
}

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
