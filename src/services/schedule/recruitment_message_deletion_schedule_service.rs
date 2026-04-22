use crate::models::entities::worker::scheduled_tasks::{self, ScheduledTaskType};
use crate::repository::schedule::{
    ScheduledTaskRecruitmentMessageDeletionRepository, ScheduledTaskRepository,
};
use crate::repository::{EnvironmentRepository, GuildEnvironmentRepository};
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sea_orm::DatabaseTransaction;
use tracing::{debug, info, warn};

/// マルチ募集投稿削除までの分数を指定する環境変数キー
pub const MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY: &str =
    "MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES";

/// 環境変数が未設定または不正な場合の既定値（7日）
pub const DEFAULT_MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES: i64 = 10080;

/// 募集投稿削除猶予時間の解決元
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionDelaySource {
    Guild,
    Global,
    Default,
}

impl DeletionDelaySource {
    fn label(self) -> &'static str {
        match self {
            Self::Guild => "ギルド環境変数",
            Self::Global => "グローバル環境変数",
            Self::Default => "固定値",
        }
    }
}

/// 解決済みの募集投稿削除猶予時間
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedDeletionDelay {
    pub minutes: i64,
    pub source: DeletionDelaySource,
}

/// 募集投稿削除スケジュール登録の抽象インターフェース
#[async_trait]
pub trait RecruitmentMessageDeletionScheduler: Send + Sync {
    /// 募集投稿削除タスクを作成する
    async fn create_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model>;

    /// 募集投稿削除タスクを現在設定で作り直す
    async fn replace_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model>;

    /// 対象募集に紐づく未実行の募集投稿削除タスクを削除する
    async fn delete_pending_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<u64>;
}

/// 募集投稿削除スケジュール管理サービス
///
/// DB上の募集データは残し、Discord上の元募集投稿だけを削除するタスクを登録する。
pub struct RecruitmentMessageDeletionScheduleService<GE, E, ST, MD>
where
    GE: GuildEnvironmentRepository,
    E: EnvironmentRepository,
    ST: ScheduledTaskRepository,
    MD: ScheduledTaskRecruitmentMessageDeletionRepository,
{
    guild_environment_repo: GE,
    environment_repo: E,
    scheduled_task_repo: ST,
    message_deletion_repo: MD,
}

#[async_trait]
impl<GE, E, ST, MD> RecruitmentMessageDeletionScheduler
    for RecruitmentMessageDeletionScheduleService<GE, E, ST, MD>
where
    GE: GuildEnvironmentRepository,
    E: EnvironmentRepository,
    ST: ScheduledTaskRepository,
    MD: ScheduledTaskRecruitmentMessageDeletionRepository,
{
    async fn create_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model> {
        RecruitmentMessageDeletionScheduleService::create_for_recruitment(
            self,
            txn,
            guild_id,
            channel_id,
            recruitment_id,
            quest_start_at,
        )
        .await
    }

    async fn replace_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model> {
        RecruitmentMessageDeletionScheduleService::replace_for_recruitment(
            self,
            txn,
            guild_id,
            channel_id,
            recruitment_id,
            quest_start_at,
        )
        .await
    }

    async fn delete_pending_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<u64> {
        RecruitmentMessageDeletionScheduleService::delete_pending_for_recruitment(
            self,
            txn,
            recruitment_id,
        )
        .await
    }
}

impl<GE, E, ST, MD> RecruitmentMessageDeletionScheduleService<GE, E, ST, MD>
where
    GE: GuildEnvironmentRepository,
    E: EnvironmentRepository,
    ST: ScheduledTaskRepository,
    MD: ScheduledTaskRecruitmentMessageDeletionRepository,
{
    pub fn new(
        guild_environment_repo: GE,
        environment_repo: E,
        scheduled_task_repo: ST,
        message_deletion_repo: MD,
    ) -> Self {
        Self {
            guild_environment_repo,
            environment_repo,
            scheduled_task_repo,
            message_deletion_repo,
        }
    }

    /// 募集投稿削除タスクを作成する
    pub async fn create_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model> {
        let resolved_delay = self.resolve_delay_minutes(txn, guild_id).await?;
        let schedule_datetime =
            calculate_deletion_schedule_datetime(quest_start_at, resolved_delay.minutes);

        let task = self
            .scheduled_task_repo
            .create(
                txn,
                schedule_datetime,
                ScheduledTaskType::RecruitmentMessageDeletion.as_i32(),
                Some(guild_id),
                Some(channel_id),
            )
            .await?;

        self.message_deletion_repo
            .create(txn, task.id, recruitment_id)
            .await?;

        info!(
            task_id = task.id,
            recruitment_id,
            guild_id,
            channel_id,
            quest_start_at = %quest_start_at,
            schedule_datetime = %schedule_datetime,
            delay_minutes = resolved_delay.minutes,
            delay_source = ?resolved_delay.source,
            "募集投稿削除タスクを作成しました"
        );

        Ok(task)
    }

    /// 募集投稿削除タスクを現在設定で作り直す
    pub async fn replace_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<scheduled_tasks::Model> {
        let deleted_count = self
            .delete_pending_for_recruitment(txn, recruitment_id)
            .await?;

        debug!(
            recruitment_id,
            deleted_count, "募集投稿削除タスクの再作成前に未実行タスクを削除しました"
        );

        self.create_for_recruitment(txn, guild_id, channel_id, recruitment_id, quest_start_at)
            .await
    }

    /// 対象募集に紐づく未実行の募集投稿削除タスクを削除する
    pub async fn delete_pending_for_recruitment(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<u64> {
        let deletions = self
            .message_deletion_repo
            .find_by_recruitment_id(txn, recruitment_id)
            .await?;

        let mut deleted_count = 0;
        for deletion in deletions {
            if let Some(task) = self
                .scheduled_task_repo
                .find_by_id(txn, deletion.task_id)
                .await?
                && task.execution_status.is_pending()
            {
                deleted_count += self.scheduled_task_repo.delete_by_id(txn, task.id).await?;
            }
        }

        Ok(deleted_count)
    }

    /// ギルド・グローバル・固定値の順で削除猶予時間を解決する
    pub async fn resolve_delay_minutes(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<ResolvedDeletionDelay> {
        let guild_value = self
            .guild_environment_repo
            .get_by_guild_and_key(
                txn,
                guild_id,
                MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY,
            )
            .await?
            .map(|env| env.value);

        if let Some(minutes) =
            parse_positive_minutes(guild_value.as_deref(), DeletionDelaySource::Guild, guild_id)
        {
            return Ok(ResolvedDeletionDelay {
                minutes,
                source: DeletionDelaySource::Guild,
            });
        }

        let global_value = self
            .environment_repo
            .get_by_key(txn, MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY)
            .await?
            .map(|env| env.value);

        Ok(resolve_delay_minutes_from_values(
            None,
            global_value.as_deref(),
            guild_id,
        ))
    }
}

/// 環境変数値から削除猶予時間を解決する
pub fn resolve_delay_minutes_from_values(
    guild_value: Option<&str>,
    global_value: Option<&str>,
    guild_id: i64,
) -> ResolvedDeletionDelay {
    if let Some(minutes) = parse_positive_minutes(guild_value, DeletionDelaySource::Guild, guild_id)
    {
        return ResolvedDeletionDelay {
            minutes,
            source: DeletionDelaySource::Guild,
        };
    }

    if let Some(minutes) =
        parse_positive_minutes(global_value, DeletionDelaySource::Global, guild_id)
    {
        return ResolvedDeletionDelay {
            minutes,
            source: DeletionDelaySource::Global,
        };
    }

    ResolvedDeletionDelay {
        minutes: DEFAULT_MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES,
        source: DeletionDelaySource::Default,
    }
}

/// 募集投稿削除タスクの実行時刻を計算する
pub fn calculate_deletion_schedule_datetime(
    quest_start_at: DateTime<Utc>,
    delay_minutes: i64,
) -> DateTime<Utc> {
    quest_start_at + Duration::minutes(delay_minutes)
}

fn parse_positive_minutes(
    value: Option<&str>,
    source: DeletionDelaySource,
    guild_id: i64,
) -> Option<i64> {
    let value = value?;
    let trimmed = value.trim();

    if trimmed.is_empty() {
        warn!(
            guild_id,
            source = source.label(),
            key = MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY,
            "募集投稿削除猶予時間が空のため次の優先順位へフォールバックします"
        );
        return None;
    }

    match trimmed.parse::<i64>() {
        Ok(minutes) if minutes > 0 => Some(minutes),
        Ok(minutes) => {
            warn!(
                guild_id,
                source = source.label(),
                key = MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY,
                value = minutes,
                "募集投稿削除猶予時間が0以下のため次の優先順位へフォールバックします"
            );
            None
        }
        Err(error) => {
            warn!(
                guild_id,
                source = source.label(),
                key = MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES_KEY,
                value = trimmed,
                error = %error,
                "募集投稿削除猶予時間を数値として解釈できないため次の優先順位へフォールバックします"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn guild値が有効ならguild分数を使う() {
        let result = resolve_delay_minutes_from_values(Some("60"), Some("120"), 1);

        assert_eq!(
            result,
            ResolvedDeletionDelay {
                minutes: 60,
                source: DeletionDelaySource::Guild
            }
        );
    }

    #[test]
    fn guild未設定ならglobal分数を使う() {
        let result = resolve_delay_minutes_from_values(None, Some("120"), 1);

        assert_eq!(
            result,
            ResolvedDeletionDelay {
                minutes: 120,
                source: DeletionDelaySource::Global
            }
        );
    }

    #[test]
    fn guild_globalとも未設定なら固定値を使う() {
        let result = resolve_delay_minutes_from_values(None, None, 1);

        assert_eq!(
            result,
            ResolvedDeletionDelay {
                minutes: DEFAULT_MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES,
                source: DeletionDelaySource::Default
            }
        );
    }

    #[test]
    fn guild値が空ならglobalへフォールバックする() {
        let result = resolve_delay_minutes_from_values(Some("  "), Some("30"), 1);

        assert_eq!(
            result,
            ResolvedDeletionDelay {
                minutes: 30,
                source: DeletionDelaySource::Global
            }
        );
    }

    #[test]
    fn 不正数値は次の優先順位へフォールバックする() {
        let result = resolve_delay_minutes_from_values(Some("abc"), Some("45"), 1);

        assert_eq!(
            result,
            ResolvedDeletionDelay {
                minutes: 45,
                source: DeletionDelaySource::Global
            }
        );
    }

    #[test]
    fn ゼロ以下は次の優先順位へフォールバックする() {
        let zero = resolve_delay_minutes_from_values(Some("0"), Some("15"), 1);
        let negative = resolve_delay_minutes_from_values(Some("-1"), Some("20"), 1);
        let default = resolve_delay_minutes_from_values(Some("0"), Some("-5"), 1);

        assert_eq!(
            zero,
            ResolvedDeletionDelay {
                minutes: 15,
                source: DeletionDelaySource::Global
            }
        );
        assert_eq!(
            negative,
            ResolvedDeletionDelay {
                minutes: 20,
                source: DeletionDelaySource::Global
            }
        );
        assert_eq!(
            default,
            ResolvedDeletionDelay {
                minutes: DEFAULT_MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES,
                source: DeletionDelaySource::Default
            }
        );
    }

    #[test]
    fn 削除タスク作成時刻は出発時刻に分数を加算する() {
        let quest_start_at = Utc.with_ymd_and_hms(2026, 4, 20, 12, 0, 0).unwrap();
        let result = calculate_deletion_schedule_datetime(quest_start_at, 90);

        assert_eq!(
            result,
            Utc.with_ymd_and_hms(2026, 4, 20, 13, 30, 0).unwrap()
        );
    }
}
