use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::models::entities::battle_styles;
use crate::types;

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

/// SeaORMを使ったBattleStyleRepository実装
pub struct SeaOrmBattleStyleRepository;

impl SeaOrmBattleStyleRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BattleStyleRepository for SeaOrmBattleStyleRepository {
    async fn get_by_id<'c, C>(
        &self,
        db: &'c C,
        id: i32,
    ) -> types::Result<Option<battle_styles::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = battle_styles::Entity::find()
            .filter(battle_styles::Column::Id.eq(id))
            .one(db)
            .await?;

        Ok(result)
    }

    async fn get_all<'c, C>(&self, db: &'c C) -> types::Result<Vec<battle_styles::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = battle_styles::Entity::find()
            .order_by_asc(battle_styles::Column::SortOrder)
            .all(db)
            .await?;

        Ok(result)
    }
}
