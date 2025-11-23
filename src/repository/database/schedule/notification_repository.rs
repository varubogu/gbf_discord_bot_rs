use crate::models::entities::notifications;
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use tracing::{debug, error};

/// 通知リポジトリ
pub struct NotificationRepository {
    db: DatabaseConnection,
}

impl NotificationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 指定した日時範囲内の通知を取得
    pub async fn find_by_datetime_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<notifications::Model>> {
        debug!(
            from = %from,
            to = %to,
            "指定範囲内の通知を取得します"
        );

        let notifications = notifications::Entity::find()
            .filter(notifications::Column::ScheduleDatetime.gte(from))
            .filter(notifications::Column::ScheduleDatetime.lt(to))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!(error = %e, "通知の取得に失敗しました");
                e
            })?;

        debug!(count = notifications.len(), "通知を取得しました");
        Ok(notifications)
    }

    /// 通知を作成（トランザクション付き）
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        schedule_datetime: DateTime<Utc>,
        guild_id: i64,
        channel_id: i64,
        message_text_id: String,
    ) -> Result<notifications::Model> {
        debug!(
            schedule_datetime = %schedule_datetime,
            guild_id = %guild_id,
            channel_id = %channel_id,
            message_text_id = %message_text_id,
            "通知を作成します"
        );

        let now = Utc::now();
        let active_model = notifications::ActiveModel {
            id: sea_orm::NotSet,
            schedule_datetime: Set(schedule_datetime),
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_text_id: Set(message_text_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "通知の作成に失敗しました");
            e
        })?;

        debug!(id = model.id, "通知を作成しました");
        Ok(model)
    }

    /// 通知を一括作成（トランザクション付き）
    pub async fn bulk_create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notifications_data: Vec<(DateTime<Utc>, i64, i64, String)>,
    ) -> Result<()> {
        debug!(count = notifications_data.len(), "通知を一括作成します");

        let now = Utc::now();
        let active_models: Vec<notifications::ActiveModel> = notifications_data
            .into_iter()
            .map(|(schedule_datetime, guild_id, channel_id, message_text_id)| {
                notifications::ActiveModel {
                    id: sea_orm::NotSet,
                    schedule_datetime: Set(schedule_datetime),
                    guild_id: Set(guild_id),
                    channel_id: Set(channel_id),
                    message_text_id: Set(message_text_id),
                    created_at: Set(now),
                    updated_at: Set(now),
                }
            })
            .collect();

        if !active_models.is_empty() {
            notifications::Entity::insert_many(active_models)
                .exec(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, "通知の一括作成に失敗しました");
                    e
                })?;
        }

        debug!("通知の一括作成が完了しました");
        Ok(())
    }

    /// ギルドIDで通知を取得
    pub async fn find_by_guild_id(&self, guild_id: i64) -> Result<Vec<notifications::Model>> {
        debug!(guild_id = %guild_id, "ギルドの通知を取得します");

        let notifications = notifications::Entity::find()
            .filter(notifications::Column::GuildId.eq(guild_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!(error = %e, "通知の取得に失敗しました");
                e
            })?;

        debug!(count = notifications.len(), "通知を取得しました");
        Ok(notifications)
    }

    /// 通知IDで通知を削除（トランザクション付き）
    pub async fn delete_by_id_with_txn(&self, txn: &DatabaseTransaction, notification_id: i32) -> Result<u64> {
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
    pub async fn delete_all_with_txn(&self, txn: &DatabaseTransaction) -> Result<u64> {
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
}
