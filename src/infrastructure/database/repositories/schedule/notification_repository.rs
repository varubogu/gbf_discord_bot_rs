use crate::models::entities::worker::{notifications, scheduled_tasks};
use crate::repository::schedule::NotificationRepository;
use crate::types::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 通知リポジトリ
#[derive(Default, Debug, Clone, Copy)]
pub struct SeaOrmNotificationRepository;

#[async_trait]
impl NotificationRepository for SeaOrmNotificationRepository {
    /// 指定した日時範囲内の未送信通知を取得（DatabaseConnection）
    async fn find_by_datetime_range_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>> {
        Self::find_by_datetime_range_internal(db, from, to).await
    }

    /// 指定した日時範囲内の未送信通知を取得（トランザクション内）
    async fn find_by_datetime_range_with_txn(
        &self,
        txn: &DatabaseTransaction,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>> {
        Self::find_by_datetime_range_internal(txn, from, to).await
    }

    /// 通知を作成（トランザクション付き）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
        guild_id: i64,
        channel_id: i64,
        message_text_id: String,
    ) -> Result<notifications::Model> {
        debug!(
            task_id = %task_id,
            guild_id = %guild_id,
            channel_id = %channel_id,
            message_text_id = %message_text_id,
            "通知を作成します"
        );

        let now = Utc::now();
        let active_model = notifications::ActiveModel {
            id: sea_orm::NotSet,
            task_id: Set(task_id),
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_text_id: Set(message_text_id),
            is_sent: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "通知の作成に失敗しました");
            e
        })?;

        debug!(id = model.id, task_id = %task_id, "通知を作成しました");
        Ok(model)
    }

    /// task_idで通知を取得（トランザクション付き）
    async fn find_by_task_id(
        &self,
        txn: &DatabaseTransaction,
        task_id: i32,
    ) -> Result<Option<notifications::Model>> {
        debug!(task_id = %task_id, "task_idで通知を取得します");

        let notification = notifications::Entity::find()
            .filter(notifications::Column::TaskId.eq(task_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, task_id = %task_id, "通知の取得に失敗しました");
                e
            })?;

        debug!(
            task_id = %task_id,
            found = notification.is_some(),
            "通知を取得しました"
        );
        Ok(notification)
    }

    // /// 通知を一括作成（トランザクション付き）
    // pub async fn bulk_create_with_txn(
    //     &self,
    //     txn: &DatabaseTransaction,
    //     notifications_data: Vec<(DateTime<Utc>, i64, i64, String)>,
    // ) -> Result<()> {
    //     debug!(count = notifications_data.len(), "通知を一括作成します");

    //     let now = Utc::now();
    //     let active_models: Vec<notifications::ActiveModel> = notifications_data
    //         .into_iter()
    //         .map(|(schedule_datetime, guild_id, channel_id, message_text_id)| {
    //             notifications::ActiveModel {
    //                 id: sea_orm::NotSet,
    //                 schedule_datetime: Set(schedule_datetime),
    //                 guild_id: Set(guild_id),
    //                 channel_id: Set(channel_id),
    //                 message_text_id: Set(message_text_id),
    //                 is_sent: Set(false),
    //                 created_at: Set(now),
    //                 updated_at: Set(now),
    //             }
    //         })
    //         .collect();

    //     if !active_models.is_empty() {
    //         notifications::Entity::insert_many(active_models)
    //             .exec(txn)
    //             .await
    //             .map_err(|e| {
    //                 error!(error = %e, "通知の一括作成に失敗しました");
    //                 e
    //             })?;
    //     }

    //     debug!("通知の一括作成が完了しました");
    //     Ok(())
    // }

    // /// ギルドIDで通知を取得
    // pub async fn find_by_guild_id(&self, guild_id: i64) -> Result<Vec<notifications::Model>> {
    //     debug!(guild_id = %guild_id, "ギルドの通知を取得します");

    //     let notifications = notifications::Entity::find()
    //         .filter(notifications::Column::GuildId.eq(guild_id))
    //         .all(&self.db)
    //         .await
    //         .map_err(|e| {
    //             error!(error = %e, "通知の取得に失敗しました");
    //             e
    //         })?;

    //     debug!(count = notifications.len(), "通知を取得しました");
    //     Ok(notifications)
    // }

    // /// ギルドの通知を取得（トランザクション内）
    // pub async fn find_by_guild_id_with_txn(
    //     &self,
    //     txn: &DatabaseTransaction,
    //     guild_id: i64,
    // ) -> Result<Vec<notifications::Model>> {
    //     debug!(guild_id = %guild_id, "ギルドの通知を取得します（トランザクション内）");

    //     let notifications = notifications::Entity::find()
    //         .filter(notifications::Column::GuildId.eq(guild_id))
    //         .all(txn)
    //         .await
    //         .map_err(|e| {
    //             error!(error = %e, "通知の取得に失敗しました");
    //             e
    //         })?;

    //     debug!(count = notifications.len(), "通知を取得しました");
    //     Ok(notifications)
    // }

    /// IDで通知を取得（トランザクション付き）
    async fn find_by_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<Option<notifications::Model>> {
        debug!(notification_id = %notification_id, "通知を取得します");

        let notification = notifications::Entity::find_by_id(notification_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, notification_id = %notification_id, "通知の取得に失敗しました");
                e
            })?;

        debug!(
            notification_id = %notification_id,
            found = notification.is_some(),
            "通知を取得しました"
        );
        Ok(notification)
    }

    /// 通知IDで通知を削除（トランザクション付き）
    async fn delete_by_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64> {
        debug!(notification_id = %notification_id, "通知を削除します");

        let result = notifications::Entity::delete_by_id(notification_id)
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, notification_id = %notification_id, "通知の削除に失敗しました");
                e
            })?;

        debug!(notification_id = %notification_id, deleted_count = result.rows_affected, "通知を削除しました");
        Ok(result.rows_affected)
    }

    /// すべての通知を削除（トランザクション付き）
    async fn delete_all_with_txn(&self, txn: &DatabaseTransaction) -> Result<u64> {
        debug!("すべての通知を削除します");

        let result = notifications::Entity::delete_many()
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, "通知の削除に失敗しました");
                e
            })?;

        debug!(deleted_count = result.rows_affected, "通知を削除しました");
        Ok(result.rows_affected)
    }

    /// 通知を送信済みとしてマーク（トランザクション付き）
    async fn mark_as_sent_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<notifications::Model> {
        debug!(notification_id = %notification_id, "通知を送信済みとしてマークします");

        // 通知を取得
        let notification = notifications::Entity::find_by_id(notification_id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, notification_id = %notification_id, "通知の取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(notification_id = %notification_id, "通知が見つかりません");
                crate::types::AppError::NotFound(format!("通知が見つかりません: {notification_id}"))
            })?;

        // is_sentをtrueに更新
        let mut active_model: notifications::ActiveModel = notification.into();
        active_model.is_sent = Set(true);
        active_model.updated_at = Set(Utc::now());

        let updated = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, notification_id = %notification_id, "通知の更新に失敗しました");
            e
        })?;

        debug!(notification_id = %notification_id, "通知を送信済みとしてマークしました");
        Ok(updated)
    }
}

impl SeaOrmNotificationRepository {
    pub fn new() -> Self {
        Self
    }

    /// 指定した日時範囲内の未送信通知を取得（内部実装）
    async fn find_by_datetime_range_internal<C>(
        db: &C,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の未送信通知を取得します"
        );

        // scheduled_tasksでフィルタリングしてから、対応するnotificationsを取得
        let tasks = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::ScheduleDatetime.gte(from))
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(to))
            .filter(scheduled_tasks::Column::TaskType.eq(1)) // Notification
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, "タスクの取得に失敗しました");
                e
            })?;

        let task_ids: Vec<i32> = tasks.into_iter().map(|t| t.id).collect();

        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let notifications = notifications::Entity::find()
            .filter(notifications::Column::TaskId.is_in(task_ids))
            .filter(notifications::Column::IsSent.eq(false))
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, "通知の取得に失敗しました");
                e
            })?;

        debug!(count = notifications.len(), "未送信通知を取得しました");
        Ok(notifications)
    }
}
