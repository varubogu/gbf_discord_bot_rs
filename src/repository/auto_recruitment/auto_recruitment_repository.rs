//! 自動募集設定リポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitments;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// 自動募集設定の作成パラメータ
#[derive(Debug, Clone)]
pub struct CreateAutoRecruitmentParams {
    pub guild_id: i64,
    pub category_id: i64,
    pub matching_channel_id: Option<i64>,
    pub quest_channel_id: Option<i64>,
    pub matching_channel_is_bot_created: bool,
    pub quest_channel_is_bot_created: bool,
    pub matching_message_id: Option<i64>,
    pub days_range: i32,
}

/// 自動募集設定リポジトリの抽象インターフェース
#[async_trait]
pub trait AutoRecruitmentRepository: Send + Sync {
    /// 全ギルドの自動募集設定を取得
    async fn find_all(&self, txn: &DatabaseTransaction) -> Result<Vec<auto_recruitments::Model>>;

    /// ギルドIDで自動募集設定を取得
    async fn find_by_guild_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Option<auto_recruitments::Model>>;

    /// 自動募集設定を作成
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        params: CreateAutoRecruitmentParams,
    ) -> Result<auto_recruitments::Model>;

    /// 募集日数を更新
    async fn update_days_range(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        days_range: i32,
    ) -> Result<auto_recruitments::Model>;

    /// マッチングチャンネルIDを更新
    async fn update_matching_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        matching_channel_id: Option<i64>,
    ) -> Result<auto_recruitments::Model>;

    /// クエストチャンネルIDを更新
    async fn update_quest_channel_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_channel_id: Option<i64>,
    ) -> Result<auto_recruitments::Model>;

    /// 自動募集設定を削除
    async fn delete(&self, txn: &DatabaseTransaction, guild_id: i64) -> Result<u64>;
}
