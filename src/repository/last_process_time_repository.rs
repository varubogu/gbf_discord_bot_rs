use crate::models::entities::worker::last_process_times::{self, LastProcessType};
use crate::models::last_process_times::LastProcessTime;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;

/// 最終処理時刻リポジトリの抽象インターフェース
#[async_trait]
pub trait LastProcessTimeRepository: Send + Sync {
    /// process_typeでlast_process_timeを取得
    async fn find_by_type<C>(
        &self,
        db: &C,
        process_type: LastProcessType,
    ) -> Result<Option<LastProcessTime>>
    where
        C: sea_orm::ConnectionTrait;

    /// スケジュール処理のlast_process_timeを取得
    async fn find_schedule_last_process_time<C>(&self, db: &C) -> Result<Option<LastProcessTime>>
    where
        C: sea_orm::ConnectionTrait;

    /// last_process_timeを更新（トランザクション付き）
    /// レコードが存在しない場合は新規作成、存在する場合は更新
    async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        process_type: LastProcessType,
        execute_time: DateTime<Utc>,
    ) -> Result<last_process_times::Model>;
}
