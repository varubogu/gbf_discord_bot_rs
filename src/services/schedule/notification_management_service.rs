use crate::repository::database::schedule::{
    NotificationRelBattleRecruitmentRepository, NotificationRepository,
};
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// 通知管理Service
/// 通知の作成・削除・リレーション管理の責務を持つ
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

    /// 募集に紐づく通知とリレーションを削除
    pub async fn delete_recruitment_notifications(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<usize> {
        use tracing::debug;

        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelBattleRecruitmentRepository::new();

        // 募集に紐づく通知を検索
        let relations = rel_repo
            .find_by_recruit_id_with_txn(txn, recruitment_id)
            .await?;

        let relations_count = relations.len();

        debug!(
            recruitment_id = recruitment_id,
            relations_count = relations_count,
            "募集に紐づく通知とリレーションを削除します"
        );

        // 外部キー制約を考慮し、リレーション→通知の順で削除
        for relation in relations {
            // リレーションを削除
            rel_repo
                .delete_by_notification_id_with_txn(txn, relation.notification_id)
                .await?;
            debug!(
                notification_id = relation.notification_id,
                "リレーションを削除しました"
            );

            // 通知を削除
            notification_repo
                .delete_by_id_with_txn(txn, relation.notification_id)
                .await?;
            debug!(
                notification_id = relation.notification_id,
                "通知を削除しました"
            );
        }

        info!(
            recruitment_id = recruitment_id,
            deleted_count = relations_count,
            "募集に紐づく通知とリレーションの削除が完了しました"
        );

        Ok(relations_count)
    }
}
