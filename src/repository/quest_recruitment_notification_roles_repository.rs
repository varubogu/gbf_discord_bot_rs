use crate::models::entities::guild_master::quest_recruitment_notification_roles;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// クエスト別募集通知ロールリポジトリの抽象インターフェース
#[async_trait]
pub trait QuestRecruitmentNotificationRolesRepository: Send + Sync {
    /// クエスト別募集通知ロールを登録（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        role_id: i64,
    ) -> Result<quest_recruitment_notification_roles::Model>;

    /// クエスト別募集通知ロールを削除（トランザクション内）
    async fn delete_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        role_id: i64,
    ) -> Result<u64>;

    /// ギルドIDとクエストIDでクエスト別募集通知ロール一覧を取得（トランザクション内、seq昇順）
    async fn find_by_guild_and_quest_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Vec<quest_recruitment_notification_roles::Model>>;

    /// ギルドIDで全クエストの募集通知ロール一覧を取得（トランザクション内、quest_id・seq昇順）
    async fn find_by_guild_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<quest_recruitment_notification_roles::Model>>;

    /// クエスト別募集通知ロールが存在するかチェック（トランザクション内）
    async fn exists_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        role_id: i64,
    ) -> Result<bool>;
}
