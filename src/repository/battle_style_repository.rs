use crate::models::entities::master::battle_styles;
use crate::types;
use async_trait::async_trait;

/// BattleStyleRepositoryトレイト
#[async_trait]
pub trait BattleStyleRepository: Send + Sync {
    /// IDからBattleStyleを取得
    async fn get_by_id<'c, C>(
        &self,
        db: &'c C,
        id: i32,
    ) -> types::Result<Option<battle_styles::Model>>
    where
        C: sea_orm::ConnectionTrait;

    /// すべてのBattleStyleを取得（sort_order順）
    async fn get_all<'c, C>(&self, db: &'c C) -> types::Result<Vec<battle_styles::Model>>
    where
        C: sea_orm::ConnectionTrait;
}
