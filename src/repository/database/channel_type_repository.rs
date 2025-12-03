use crate::models::entities::channel_types;
use crate::types::Result;
use sea_orm::EntityTrait;
use tracing::{debug, error};

/// channel_typesテーブルのRepository
pub struct ChannelTypeRepository;

impl ChannelTypeRepository {
    pub fn new() -> Self {
        Self
    }

    /// すべてのチャンネル種別を取得
    pub async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<channel_types::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!("すべてのチャンネル種別を取得します");

        let channel_types = channel_types::Entity::find()
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, "チャンネル種別の取得に失敗しました");
                e
            })?;

        debug!(count = channel_types.len(), "チャンネル種別を取得しました");
        Ok(channel_types)
    }

    /// IDでチャンネル種別を取得
    pub async fn get_by_id<'c, C>(&self, db: &'c C, id: i32) -> Result<Option<channel_types::Model>>
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
