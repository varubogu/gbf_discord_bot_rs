use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::models::entities::battle_styles;
use crate::types;

/// BattleStyleRepositoryトレイト
#[async_trait]
pub trait BattleStyleRepository: Send + Sync {
    /// IDからBattleStyleを取得
    async fn get_by_id(&self, id: i32) -> types::Result<Option<battle_styles::Model>>;

    /// すべてのBattleStyleを取得（sort_order順）
    async fn get_all(&self) -> types::Result<Vec<battle_styles::Model>>;
}

/// SeaORMを使ったBattleStyleRepository実装
pub struct SeaOrmBattleStyleRepository {
    conn: DatabaseConnection,
}

impl SeaOrmBattleStyleRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl BattleStyleRepository for SeaOrmBattleStyleRepository {
    async fn get_by_id(&self, id: i32) -> types::Result<Option<battle_styles::Model>> {
        let result = battle_styles::Entity::find()
            .filter(battle_styles::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(result)
    }

    async fn get_all(&self) -> types::Result<Vec<battle_styles::Model>> {
        let result = battle_styles::Entity::find()
            .order_by_asc(battle_styles::Column::SortOrder)
            .all(&self.conn)
            .await?;

        Ok(result)
    }
}
