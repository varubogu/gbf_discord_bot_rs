use crate::models::entities::{battle_types, battle_types::Entity as BattleTypeEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleType {
    pub id: i32,
    pub display_name: String,
    pub reactions: Option<String>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<battle_types::Model> for BattleType {
    fn from(model: battle_types::Model) -> Self {
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
    pub async fn get_battle_types(&self) -> Result<Vec<BattleType>, DbErr> {
        let models = BattleTypeEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_battle_type_by_id(&self, id: i32) -> Result<Option<BattleType>, DbErr> {
        let battle_type = BattleTypeEntity::find()
            .filter(battle_types::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(battle_type.map(|bt| bt.into()))
    }

    pub async fn get_battle_type_by_display_name(
        &self,
        display_name: &str,
    ) -> Result<Option<BattleType>, DbErr> {
        let battle_type = BattleTypeEntity::find()
            .filter(battle_types::Column::DisplayName.eq(display_name))
            .one(&self.conn)
            .await?;

        Ok(battle_type.map(|bt| bt.into()))
    }
}
