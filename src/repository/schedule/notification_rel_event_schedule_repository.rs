use crate::models::entities::worker::notification_rel_event_schedules;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

/// notification_rel_event_schedulesリポジトリの抽象インターフェース
#[async_trait]
pub trait NotificationRelEventScheduleRepository: Send + Sync {
    /// リレーションを作成（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        event_schedule_id: Uuid,
        event_schedule_detail_id: Uuid,
        notification_id: i32,
    ) -> Result<notification_rel_event_schedules::Model>;

    /// すべてのリレーションを削除（トランザクション内）
    async fn delete_all_with_txn(&self, txn: &DatabaseTransaction) -> Result<u64>;
}
