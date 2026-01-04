use crate::models::entities::master::channel_types;
use crate::types::Result;
use async_trait::async_trait;

/// チャンネル種別リポジトリの抽象インターフェース
#[async_trait]
pub trait ChannelTypeRepository: Send + Sync {
    /// すべてのチャンネル種別を取得
    async fn get_all<C>(&self, db: &C) -> Result<Vec<channel_types::Model>>
    where
        C: sea_orm::ConnectionTrait;

    /// IDでチャンネル種別を取得
    async fn get_by_id<C>(&self, db: &C, id: i32) -> Result<Option<channel_types::Model>>
    where
        C: sea_orm::ConnectionTrait;
}
