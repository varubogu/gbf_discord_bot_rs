use crate::repository::database::schedule::{
    NotificationRelBattleRecruitmentRepository, NotificationRepository,
};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// 通知管理Service
/// 通知の作成・リレーション作成の責務を持つ
pub struct NotificationManagementService;

impl NotificationManagementService {
    pub fn new() -> Self {
        Self
    }

    /// 募集の出発通知を作成し、募集とのリレーションを作成
    pub async fn create_recruitment_departure_notification(
        &self,
        txn: &DatabaseTransaction,
        notify_time: DateTime<Utc>,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
    ) -> Result<()> {
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelBattleRecruitmentRepository::new();

        // 通知を作成
        let notification = notification_repo
            .create_with_txn(
                txn,
                notify_time,
                guild_id,
                channel_id,
                "MSG00033".to_string(),
            )
            .await?;

        info!("募集の出発通知を登録しました");

        // 通知と募集のリレーションを作成
        rel_repo
            .create_with_txn(txn, recruitment_id, notification.id)
            .await?;

        info!("募集と通知のリレーションを登録しました");

        Ok(())
    }
}
