use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::database::schedule::{
    SeaOrmNotificationRelBattleRecruitmentRepository, SeaOrmNotificationRepository,
    SeaOrmScheduledTaskRepository,
};
use crate::repository::schedule::{
    NotificationRelBattleRecruitmentRepository, NotificationRepository, ScheduledTaskRepository,
};
use crate::services::message::MessageTextId;
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// 通知管理Service
/// 通知の作成・削除・リレーション管理の責務を持つ
pub struct NotificationManagementService;

impl Default for NotificationManagementService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManagementService {
    pub fn new() -> Self {
        Self
    }

    /// 募集の出発通知を作成し、募集とのリレーションを作成
    ///
    /// # 通知タイミング
    /// - 出発5分前: 現在時刻より5分前の時刻が未来の場合のみ作成
    /// - 出発時刻ちょうど: 必ず作成
    pub async fn create_recruitment_departure_notification(
        &self,
        txn: &DatabaseTransaction,
        departure_time: DateTime<Utc>,
        guild_id: i64,
        channel_id: i64,
        recruitment_id: i32,
    ) -> Result<()> {
        let notification_repo = SeaOrmNotificationRepository::new();
        let rel_repo = SeaOrmNotificationRelBattleRecruitmentRepository::new();
        let scheduled_task_repo = SeaOrmScheduledTaskRepository::new();

        let now = Utc::now();
        let five_minutes_before = departure_time - chrono::Duration::minutes(5);

        // 5分前通知: 現在時刻より未来の場合のみ作成
        if five_minutes_before > now {
            // 1. scheduled_taskを作成（task_type=1: Notification）
            let scheduled_task = scheduled_task_repo
                .create(
                    txn,
                    five_minutes_before,
                    ScheduledTaskType::Notification.as_i32(),
                    Some(guild_id),
                    Some(channel_id),
                )
                .await?;

            // 2. notificationを作成（task_idを指定）
            let notification = notification_repo
                .create_with_txn(
                    txn,
                    scheduled_task.id,
                    five_minutes_before,
                    guild_id,
                    channel_id,
                    MessageTextId::RecruitmentNotificationBefore5Minutes
                        .as_str()
                        .to_string(),
                )
                .await?;

            // 3. notification_relを作成
            rel_repo
                .create_with_txn(txn, recruitment_id, notification.id)
                .await?;

            info!("募集の出発5分前通知を登録しました");
        } else {
            info!("募集の出発5分前は過ぎているため、5分前通知をスキップしました");
        }

        // 出発時刻ちょうどの通知: 必ず作成
        // 1. scheduled_taskを作成（task_type=1: Notification）
        let scheduled_task = scheduled_task_repo
            .create(
                txn,
                departure_time,
                ScheduledTaskType::Notification.as_i32(),
                Some(guild_id),
                Some(channel_id),
            )
            .await?;

        // 2. notificationを作成（task_idを指定）
        let notification = notification_repo
            .create_with_txn(
                txn,
                scheduled_task.id,
                departure_time,
                guild_id,
                channel_id,
                MessageTextId::RecruitmentNotificationStart
                    .as_str()
                    .to_string(),
            )
            .await?;

        // 3. notification_relを作成
        rel_repo
            .create_with_txn(txn, recruitment_id, notification.id)
            .await?;

        info!("募集の出発時刻ちょうどの通知を登録しました");

        Ok(())
    }

    /// 募集に紐づく通知とリレーションを削除
    pub async fn delete_recruitment_notifications(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<usize> {
        use tracing::debug;

        let rel_repo = SeaOrmNotificationRelBattleRecruitmentRepository::new();
        let scheduled_task_repo = SeaOrmScheduledTaskRepository::new();

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

        // 外部キー制約を考慮し、rel → scheduled_task の順で削除
        // (scheduled_tasks削除でnotificationsもCASCADE削除される)
        for relation in relations {
            // リレーションを削除
            rel_repo
                .delete_by_notification_id_with_txn(txn, relation.notification_id)
                .await?;
            debug!(
                notification_id = relation.notification_id,
                "リレーションを削除しました"
            );

            // notification_idからscheduled_tasksテーブルを検索してtask_idを取得
            use crate::models::entities::worker::notifications;
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

            if let Some(notification) = notifications::Entity::find()
                .filter(notifications::Column::Id.eq(relation.notification_id))
                .one(txn)
                .await?
            {
                // scheduled_taskを削除（CASCADE で notifications も削除される）
                scheduled_task_repo
                    .delete_by_id(txn, notification.task_id)
                    .await?;
                debug!(
                    task_id = notification.task_id,
                    notification_id = notification.id,
                    "scheduled_taskと通知を削除しました"
                );
            }
        }

        info!(
            recruitment_id = recruitment_id,
            deleted_count = relations_count,
            "募集に紐づく通知とリレーションの削除が完了しました"
        );

        Ok(relations_count)
    }
}
