use crate::models::entities::worker::notification_rel_event_schedules;
use crate::repository::schedule::NotificationRelEventScheduleRepository;
use crate::types::Result;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, Set};
use uuid::Uuid;

/// notification_rel_event_schedulesテーブルのRepository
#[derive(Default, Debug, Clone, Copy)]
pub struct SeaOrmNotificationRelEventScheduleRepository;

#[async_trait]
impl NotificationRelEventScheduleRepository for SeaOrmNotificationRelEventScheduleRepository {
    // /// 通知IDからイベントスケジュールとの関連を取得
    // pub async fn find_by_notification_id(
    //     &self,
    //     notification_id: i32,
    // ) -> Result<Option<notification_rel_event_schedules::Model>> {
    //     let result = notification_rel_event_schedules::Entity::find()
    //         .filter(notification_rel_event_schedules::Column::NotificationId.eq(notification_id))
    //         .one(&self.db)
    //         .await?;

    //     Ok(result)
    // }

    // /// イベントスケジュールIDから通知との関連を取得
    // pub async fn find_by_event_schedule_id(
    //     &self,
    //     event_schedule_id: Uuid,
    // ) -> Result<Vec<notification_rel_event_schedules::Model>> {
    //     let results = notification_rel_event_schedules::Entity::find()
    //         .filter(notification_rel_event_schedules::Column::EventScheduleId.eq(event_schedule_id))
    //         .all(&self.db)
    //         .await?;

    //     Ok(results)
    // }

    /// リレーションを作成（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        event_schedule_id: Uuid,
        event_schedule_detail_id: Uuid,
        notification_id: i32,
    ) -> Result<notification_rel_event_schedules::Model> {
        let active_model = notification_rel_event_schedules::ActiveModel {
            event_schedule_id: Set(event_schedule_id),
            event_schedule_detail_id: Set(Some(event_schedule_detail_id)),
            notification_id: Set(notification_id),
            created_at: Set(Utc::now()),
        };

        let model = active_model.insert(txn).await?;
        Ok(model)
    }

    // /// 一括でリレーションを作成（トランザクション内）
    // pub async fn bulk_create_with_txn(
    //     &self,
    //     txn: &DatabaseTransaction,
    //     relations: Vec<(Uuid, Uuid, i32)>, // (event_schedule_id, event_schedule_detail_id, notification_id)
    // ) -> Result<()> {
    //     for (event_schedule_id, event_schedule_detail_id, notification_id) in relations {
    //         self.create_with_txn(txn, event_schedule_id, event_schedule_detail_id, notification_id).await?;
    //     }
    //     Ok(())
    // }

    /// すべてのリレーションを削除（トランザクション内）
    async fn delete_all_with_txn(&self, txn: &DatabaseTransaction) -> Result<u64> {
        let result = notification_rel_event_schedules::Entity::delete_many()
            .exec(txn)
            .await?;

        Ok(result.rows_affected)
    }
}

impl SeaOrmNotificationRelEventScheduleRepository {
    pub fn new() -> Self {
        Self
    }
}
