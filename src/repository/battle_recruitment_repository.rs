use crate::infrastructure::database::Transaction;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::types::PoiseError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// バトル募集リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait BattleRecruitmentRepository: Send + Sync {
    /// 新規募集を作成
    async fn create(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError>;

    /// メッセージIDで募集を取得
    async fn get_by_message(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
    ) -> Result<Option<BattleRecruitment>, PoiseError>;

    /// 募集終了メッセージを更新
    async fn set_end_message(&self, recruitment_id: i32, message_id: i64)
    -> Result<(), PoiseError>;

    // トランザクション対応メソッド

    /// 新規募集を作成（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &Transaction,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError>;

    /// メッセージIDで募集を取得（トランザクション内）
    async fn get_by_message_with_txn(
        &self,
        txn: &Transaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitment>, PoiseError>;

    /// 募集終了メッセージを更新（トランザクション内）
    async fn set_end_message_with_txn(
        &self,
        txn: &Transaction,
        recruitment_id: i32,
        message_id: i64,
    ) -> Result<(), PoiseError>;
}
