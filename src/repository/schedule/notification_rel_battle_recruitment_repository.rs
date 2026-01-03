use crate::models::entities::worker::notification_rel_battle_recruitments;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// notification_rel_battle_recruitmentsリポジトリの抽象インターフェース
#[async_trait]
pub trait NotificationRelBattleRecruitmentRepository: Send + Sync {
    /// 通知IDからマルチ募集との関連を取得
    async fn find_by_notification_id<C>(
        &self,
        db: &C,
        notification_id: i32,
    ) -> Result<Option<notification_rel_battle_recruitments::Model>>
    where
        C: sea_orm::ConnectionTrait;

    /// 募集IDから通知との関連を取得（トランザクション内）
    async fn find_by_recruit_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
    ) -> Result<Vec<notification_rel_battle_recruitments::Model>>;

    /// リレーションを作成（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruit_id: i32,
        notification_id: i32,
    ) -> Result<notification_rel_battle_recruitments::Model>;

    /// 通知IDに紐づくリレーションを削除（トランザクション内）
    async fn delete_by_notification_id_with_txn(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<u64>;
}
