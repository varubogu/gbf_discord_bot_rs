use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "battle_types")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub reactions: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::battle_recruitments::Entity")]
    BattleRecruitment,
    #[sea_orm(has_many = "super::quests::Entity")]
    Quest,
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl Related<super::quests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
