use crate::types::Result;
use async_trait::async_trait;

/// 募集参加者リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait RecruitmentParticipantsRepository: Send + Sync + std::fmt::Debug {
    /// 参加レコードを挿入（トランザクション対応）
    /// 既に存在する場合は何もせず、false を返す
    async fn insert_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool>;

    /// 特定属性の参加を削除（トランザクション対応）
    /// 削除された場合は true、該当レコードがない場合は false を返す
    async fn delete_by_element_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool>;

    /// ユーザーの全参加を削除（トランザクション対応）
    /// 削除されたレコード数を返す
    async fn delete_all_by_user_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<u64>;

    /// 特定の参加レコードを取得（トランザクション対応）
    async fn get_by_user_and_element<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool>
    where
        C: sea_orm::ConnectionTrait;

    /// 募集に参加しているユニークユーザー数を取得（トランザクション対応）
    async fn count_unique_users<'c, C>(&self, db: &'c C, recruitment_id: i32) -> Result<i64>
    where
        C: sea_orm::ConnectionTrait;

    /// ユーザーが参加している属性のリストを取得（トランザクション対応）
    async fn get_user_elements<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<Vec<Option<i32>>>
    where
        C: sea_orm::ConnectionTrait;

    /// 募集に参加している全ユーザーのIDリストを取得（重複なし）
    async fn get_all_participant_user_ids<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
    ) -> Result<Vec<u64>>
    where
        C: sea_orm::ConnectionTrait;
}
