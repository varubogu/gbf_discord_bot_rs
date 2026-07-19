use crate::models::entities::guild_master::auto_recruitments;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::models::entities::worker::{quest_matching_users, scheduled_tasks};
use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::types::Result;
use chrono::{Duration, Utc};
use sea_orm::DatabaseTransaction;
use tracing::debug;
use uuid::Uuid;

/// 自動マッチングタスクディスパッチが必要とする scheduled_tasks / auto_recruitments /
/// quest_matchings / quest_matching_users / quests への直接アクセスを集約する薄いサービス。
///
/// facade層がrepositoryを直接呼ばずに済むよう、ディスパッチ処理専用の窓口として存在する。
pub struct AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>
where
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository,
{
    task_repo: ST,
    auto_recruitment_repo: ARR,
    matching_repo: QMR,
    matching_user_repo: QMUR,
    quest_repo: Q,
}

impl<ST, ARR, QMR, QMUR, Q> AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>
where
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository,
{
    pub fn new(
        task_repo: ST,
        auto_recruitment_repo: ARR,
        matching_repo: QMR,
        matching_user_repo: QMUR,
        quest_repo: Q,
    ) -> Self {
        Self {
            task_repo,
            auto_recruitment_repo,
            matching_repo,
            matching_user_repo,
            quest_repo,
        }
    }

    pub async fn find_task(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<scheduled_tasks::Model>> {
        self.task_repo.find_by_id(txn, task_id).await
    }

    pub async fn mark_succeeded(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
        self.task_repo.mark_as_succeeded(txn, task_id).await?;
        Ok(())
    }

    /// 次回実行タスクを作成（10秒後）
    pub async fn register_next_scheduled_task(&self, txn: &DatabaseTransaction) -> Result<i32> {
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

    pub async fn find_auto_recruitment_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<auto_recruitments::Model>> {
        self.auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await
    }

    pub async fn find_active_matching_users(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
    ) -> Result<Vec<quest_matching_users::Model>> {
        self.matching_user_repo
            .find_active_by_matching(txn, guild_id, matching_id)
            .await
    }

    pub async fn find_quest(
        &self,
        txn: &DatabaseTransaction,
        quest_id: i32,
    ) -> Result<Option<Quest>> {
        self.quest_repo.get_by_target_id(txn, quest_id).await
    }

    pub async fn set_matching_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_id: Uuid,
        recruitment_id: i32,
    ) -> Result<()> {
        self.matching_repo
            .set_recruitment_id(txn, guild_id, matching_id, recruitment_id)
            .await?;
        Ok(())
    }
}
