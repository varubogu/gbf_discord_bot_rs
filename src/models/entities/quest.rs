use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub target_id: i32,
    pub quest_name: String,
    pub default_battle_type: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::quest_alias::Entity")]
    QuestAlias,
}

impl Related<super::quest_alias::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::QuestAlias.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}