use crate::models::entities::master::battle_styles::{self, Entity as BattleStyleEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleStyle {
    pub id: i32,
    pub display_name: String,
    pub reactions: Option<String>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<battle_styles::Model> for BattleStyle {
    fn from(model: battle_styles::Model) -> Self {
        Self {
            id: model.id,
            display_name: model.display_name,
            reactions: model.reactions,
            sort_order: model.sort_order,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_battle_styles(&self) -> Result<Vec<BattleStyle>, DbErr> {
        let models = BattleStyleEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_battle_style_by_id(&self, id: i32) -> Result<Option<BattleStyle>, DbErr> {
        let battle_style = BattleStyleEntity::find()
            .filter(battle_styles::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(battle_style.map(|bt| bt.into()))
    }

    pub async fn get_battle_style_by_display_name(
        &self,
        display_name: &str,
    ) -> Result<Option<BattleStyle>, DbErr> {
        let battle_style = BattleStyleEntity::find()
            .filter(battle_styles::Column::DisplayName.eq(display_name))
            .one(&self.conn)
            .await?;

        Ok(battle_style.map(|bt| bt.into()))
    }
}
