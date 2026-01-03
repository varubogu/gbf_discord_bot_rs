use crate::models::entities::worker::battle_recruitment_dismissals;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// マルチ募集解散リポジトリの抽象インターフェース
#[async_trait]
pub trait BattleRecruitmentDismissalRepository: Send + Sync {
    /// 解散時刻を作成（絶対日時）
    async fn create_absolute(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        input_value: String,
        dismissal_datetime: DateTime<Utc>,
    ) -> Result<battle_recruitment_dismissals::Model>;

    /// 解散時刻を作成（相対時刻）
    async fn create_relative(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        input_value: String,
        relative_days: i32,
        relative_hours: i32,
        relative_minutes: i32,
    ) -> Result<battle_recruitment_dismissals::Model>;

    /// recruitment_idで解散時刻を取得
    async fn find_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<battle_recruitment_dismissals::Model>>;

    /// idで解散時刻を取得
    async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
    ) -> Result<Option<battle_recruitment_dismissals::Model>>;

    /// recruitment_idで解散時刻を削除
    async fn delete_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<u64>;
}
