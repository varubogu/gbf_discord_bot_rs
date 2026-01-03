use crate::models::entities::guild_master::battle_recruitment_schedule_dismissals;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// 定期募集解散リポジトリの抽象インターフェース
#[async_trait]
pub trait BattleRecruitmentScheduleDismissalRepository: Send + Sync {
    /// 解散時刻を作成（絶対時刻）
    async fn create_absolute(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
        input_value: String,
        dismissal_time: sea_orm::entity::prelude::TimeTime,
    ) -> Result<battle_recruitment_schedule_dismissals::Model>;

    /// 解散時刻を作成（相対時刻）
    async fn create_relative(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
        input_value: String,
        relative_days: i32,
        relative_hours: i32,
        relative_minutes: i32,
    ) -> Result<battle_recruitment_schedule_dismissals::Model>;

    /// schedule_idで解散時刻を取得
    async fn find_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<Vec<battle_recruitment_schedule_dismissals::Model>>;

    /// schedule_idで解散時刻を削除
    async fn delete_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<u64>;
}
