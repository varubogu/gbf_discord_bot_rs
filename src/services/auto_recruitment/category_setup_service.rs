use crate::models::entities::guild_master::{
    auto_recruitment_channels, auto_recruitment_quest_messages, auto_recruitments,
};
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentQuestMessageRepository,
    AutoRecruitmentRepository, CreateAutoRecruitmentParams, QuestMatchingRepository,
    QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::types::{AppError, Result};
use chrono::{Datelike, Duration, Utc};
use sea_orm::DatabaseTransaction;
use std::collections::HashSet;
use tracing::info;

/// 自動募集カテゴリ設定サービス
///
/// Facade層からRepository直接依存を取り除くため、
/// カテゴリ設定に関する永続化操作を集約する。
pub struct CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>
where
    AR: AutoRecruitmentRepository,
    AC: AutoRecruitmentChannelRepository,
    Q: QuestRepository,
    AQM: AutoRecruitmentQuestMessageRepository,
    QMU: QuestMatchingUserRepository,
    QM: QuestMatchingRepository,
    ST: ScheduledTaskRepository,
{
    auto_recruitment_repo: AR,
    auto_recruitment_channel_repo: AC,
    quest_repo: Q,
    quest_message_repo: AQM,
    quest_matching_user_repo: QMU,
    quest_matching_repo: QM,
    scheduled_task_repo: ST,
}

impl<AR, AC, Q, AQM, QMU, QM, ST> CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>
where
    AR: AutoRecruitmentRepository,
    AC: AutoRecruitmentChannelRepository,
    Q: QuestRepository,
    AQM: AutoRecruitmentQuestMessageRepository,
    QMU: QuestMatchingUserRepository,
    QM: QuestMatchingRepository,
    ST: ScheduledTaskRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auto_recruitment_repo: AR,
        auto_recruitment_channel_repo: AC,
        quest_repo: Q,
        quest_message_repo: AQM,
        quest_matching_user_repo: QMU,
        quest_matching_repo: QM,
        scheduled_task_repo: ST,
    ) -> Self {
        Self {
            auto_recruitment_repo,
            auto_recruitment_channel_repo,
            quest_repo,
            quest_message_repo,
            quest_matching_user_repo,
            quest_matching_repo,
            scheduled_task_repo,
        }
    }

    /// 自動募集が未登録であることを確認
    pub async fn ensure_not_registered(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<()> {
        if self
            .auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?
            .is_some()
        {
            return Err(AppError::Business {
                message: "このギルドには既に自動募集が登録されています。先に解除してください。"
                    .to_string(),
            });
        }
        Ok(())
    }

    /// 自動募集設定を取得（未登録時はエラー）
    pub async fn get_auto_recruitment_or_err(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<auto_recruitments::Model> {
        self.auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })
    }

    /// 有効なクエスト一覧を取得
    pub async fn get_enabled_quests(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<Quest>> {
        let enabled_quest_results = self
            .quest_repo
            .search_enabled_quests(txn, guild_id, "")
            .await?;
        let enabled_quest_ids: HashSet<i32> =
            enabled_quest_results.iter().map(|q| q.quest_id).collect();

        let all_quests = self.quest_repo.get_all(txn).await?;
        Ok(all_quests
            .into_iter()
            .filter(|q| enabled_quest_ids.contains(&q.id))
            .collect())
    }

    /// 自動募集設定レコードを作成
    #[allow(clippy::too_many_arguments)]
    pub async fn create_auto_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        category_id: i64,
        matching_channel_id: Option<i64>,
        quest_channel_id: Option<i64>,
        matching_channel_is_bot_created: bool,
        quest_channel_is_bot_created: bool,
        matching_message_id: Option<i64>,
        days_range: i32,
    ) -> Result<auto_recruitments::Model> {
        self.auto_recruitment_repo
            .create(
                txn,
                CreateAutoRecruitmentParams {
                    guild_id,
                    category_id,
                    matching_channel_id,
                    quest_channel_id,
                    matching_channel_is_bot_created,
                    quest_channel_is_bot_created,
                    matching_message_id,
                    days_range,
                },
            )
            .await
    }

    /// 日時チャンネルレコードを作成
    #[allow(clippy::too_many_arguments)]
    pub async fn create_date_channel(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        month: i32,
        day: i32,
        sort_order: i32,
        is_bot_created: bool,
        message_id: Option<i64>,
    ) -> Result<auto_recruitment_channels::Model> {
        self.auto_recruitment_channel_repo
            .create(
                txn,
                guild_id,
                channel_id,
                month,
                day,
                sort_order,
                is_bot_created,
                message_id,
            )
            .await
    }

    /// 日時チャンネル一覧を取得
    pub async fn find_date_channels(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_channels::Model>> {
        self.auto_recruitment_channel_repo
            .find_by_guild_id(txn, guild_id)
            .await
    }

    /// 日時チャンネルレコードを削除
    pub async fn delete_date_channel_by_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<u64> {
        self.auto_recruitment_channel_repo
            .delete_by_channel_id(txn, guild_id, channel_id)
            .await
    }

    /// 日時チャンネルレコードを全削除
    pub async fn delete_all_date_channels(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<u64> {
        self.auto_recruitment_channel_repo
            .delete_all_by_guild_id(txn, guild_id)
            .await
    }

    /// クエストメッセージを作成または更新
    pub async fn upsert_quest_message(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        message_id: i64,
    ) -> Result<auto_recruitment_quest_messages::Model> {
        self.quest_message_repo
            .upsert(txn, guild_id, quest_id, message_id)
            .await
    }

    /// クエストメッセージ一覧を取得
    pub async fn find_quest_messages(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_quest_messages::Model>> {
        self.quest_message_repo
            .find_all_by_guild(txn, guild_id)
            .await
    }

    /// クエストメッセージを全削除
    pub async fn delete_all_quest_messages(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<u64> {
        self.quest_message_repo
            .delete_all_by_guild(txn, guild_id)
            .await
    }

    /// マッチング関連データを全削除
    pub async fn delete_all_matching_data(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<()> {
        self.quest_matching_user_repo
            .delete_all_by_guild(txn, guild_id)
            .await?;
        self.quest_matching_repo
            .delete_all_by_guild(txn, guild_id)
            .await?;
        Ok(())
    }

    /// 自動募集設定を削除
    pub async fn delete_auto_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<u64> {
        self.auto_recruitment_repo.delete(txn, guild_id).await
    }

    /// 募集日数を更新
    pub async fn update_days_range(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        days_range: i32,
    ) -> Result<auto_recruitments::Model> {
        self.auto_recruitment_repo
            .update_days_range(txn, guild_id, days_range)
            .await
    }

    /// 初期ローテーションタスクを必要時のみ作成
    pub async fn ensure_initial_rotation_task(&self, txn: &DatabaseTransaction) -> Result<()> {
        let now_utc = Utc::now();
        let now_jst = now_utc + Duration::hours(9);
        let tomorrow_jst = (now_jst + Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::Business {
                message: "ローテーションタスクの日時生成に失敗しました".to_string(),
            })?;
        let next_execution_utc = tomorrow_jst - Duration::hours(9);
        let next_execution =
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(next_execution_utc, Utc);

        let pending_tasks = self
            .scheduled_task_repo
            .find_pending_to(txn, next_execution + Duration::days(1))
            .await?;

        let has_rotation_task = pending_tasks
            .iter()
            .any(|t| t.task_type == ScheduledTaskType::AutoRecruitmentRotation as i32);

        if !has_rotation_task {
            self.scheduled_task_repo
                .create(
                    txn,
                    next_execution,
                    ScheduledTaskType::AutoRecruitmentRotation as i32,
                    None,
                    None,
                )
                .await?;
            info!(
                next_execution = %next_execution,
                "初期ローテーションタスクを作成しました"
            );
        }

        Ok(())
    }

    /// 初期自動マッチングタスクを必要時のみ作成
    pub async fn ensure_initial_auto_matching_task(&self, txn: &DatabaseTransaction) -> Result<()> {
        let next_execution = Utc::now() + Duration::seconds(10);

        let pending_tasks = self
            .scheduled_task_repo
            .find_pending_to(txn, next_execution + Duration::minutes(1))
            .await?;

        let has_matching_task = pending_tasks
            .iter()
            .any(|t| t.task_type == ScheduledTaskType::AutoMatching as i32);

        if !has_matching_task {
            self.scheduled_task_repo
                .create(
                    txn,
                    next_execution,
                    ScheduledTaskType::AutoMatching as i32,
                    None,
                    None,
                )
                .await?;
            info!(
                next_execution = %next_execution,
                "初期自動マッチングタスクを作成しました"
            );
        }

        Ok(())
    }

    /// JST基準の今日の日付を返す
    pub fn today_jst() -> chrono::NaiveDate {
        let now_jst = Utc::now() + Duration::hours(9);
        now_jst.date_naive()
    }

    /// 月日をJST日付へ変換（失敗時は今日を返す）
    pub fn to_jst_date_or_today(month: i32, day: i32) -> chrono::NaiveDate {
        let today = Self::today_jst();
        chrono::NaiveDate::from_ymd_opt(today.year(), month as u32, day as u32).unwrap_or(today)
    }
}
