use crate::models::entities::notification_rel_battle_recruitments;
use crate::types::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};

/// notification_rel_battle_recruitmentsテーブルのRepository
pub struct NotificationRelBattleRecruitmentRepository;

impl NotificationRelBattleRecruitmentRepository {
    pub fn new() -> Self {
        Self
    }

    /// 通知IDからマルチ募集との関連を取得
    pub async fn find_by_notification_id<'c, C>(
        &self,
        db: &'c C,
        notification_id: i32,
    ) -> Result<Option<notification_rel_battle_recruitments::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = notification_rel_battle_recruitments::Entity::find()
            .filter(
                notification_rel_battle_recruitments::Column::NotificationId.eq(notification_id),
            )
            .one(db)
            .await?;

        Ok(result)
    }

    // /// 募集IDから通知との関連を取得
    // pub async fn find_by_recruit_id(
    //     &self,
    //     recruit_id: i32,
    // ) -> Result<Vec<notification_rel_battle_recruitments::Model>> {
    //     let results = notification_rel_battle_recruitments::Entity::find()
    //         .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruit_id))
    //         .all(&self.db)
    //         .await?;

    //     Ok(results)
    // }

    /// 募集IDから通知との関連を取得（トランザクション内）
    pub async fn find_by_recruit_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<Vec<notification_rel_battle_recruitments::Model>> {
        let results = notification_rel_battle_recruitments::Entity::find()
            .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruit_id))
            .all(txn)
            .await?;

        Ok(results)
    }

    /// リレーションを作成（トランザクション内）
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
        notification_id: i32,
    ) -> Result<notification_rel_battle_recruitments::Model> {
        let active_model = notification_rel_battle_recruitments::ActiveModel {
            recruit_id: Set(recruit_id),
            notification_id: Set(notification_id),
            created_at: Set(Utc::now()),
        };

        let model = active_model.insert(txn).await?;
        Ok(model)
    }

    /// 通知IDに紐づくリレーションを削除（トランザクション内）
    pub async fn delete_by_notification_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64> {
        let result = notification_rel_battle_recruitments::Entity::delete_many()
            .filter(
                notification_rel_battle_recruitments::Column::NotificationId.eq(notification_id),
            )
            .exec(txn)
            .await?;

        Ok(result.rows_affected)
    }

    // /// 募集IDに紐づくリレーションを削除（トランザクション内）
    // pub async fn delete_by_recruit_id_with_txn(
    //     &self,
    //     txn: &DatabaseTransaction,
    //     recruit_id: i32,
    // ) -> Result<u64> {
    //     let result = notification_rel_battle_recruitments::Entity::delete_many()
    //         .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruit_id))
    //         .exec(txn)
    //         .await?;

    //     Ok(result.rows_affected)
    // }
}
