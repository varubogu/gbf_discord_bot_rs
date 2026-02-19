use crate::models::entities::master::channel_types;
use crate::repository::ChannelTypeRepository;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::EntityTrait;
use tracing::{debug, error};

/// channel_typesテーブルのRepository
#[derive(Default, Debug, Clone, Copy)]
pub struct SeaOrmChannelTypeRepository;

impl SeaOrmChannelTypeRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ChannelTypeRepository for SeaOrmChannelTypeRepository {
    /// すべてのチャンネル種別を取得
    async fn get_all<C>(&self, db: &C) -> Result<Vec<channel_types::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!("すべてのチャンネル種別を取得します");

        let channel_types = channel_types::Entity::find().all(db).await.map_err(|e| {
            error!(error = %e, "チャンネル種別の取得に失敗しました");
            e
        })?;

        debug!(count = channel_types.len(), "チャンネル種別を取得しました");
        Ok(channel_types)
    }

    /// IDでチャンネル種別を取得
    async fn get_by_id<C>(&self, db: &C, id: i32) -> Result<Option<channel_types::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!(id = id, "チャンネル種別を取得します");

        let channel_type = channel_types::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| {
                error!(error = %e, id = id, "チャンネル種別の取得に失敗しました");
                e
            })?;

        Ok(channel_type)
    }
}
