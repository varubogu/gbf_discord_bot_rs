use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub default_battle_style: i32,
    pub recruit_count: i32,
    pub available_battle_styles: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::quest_aliases::Entity")]
    QuestAlias,
    #[sea_orm(
        belongs_to = "super::battle_types::Entity",
        from = "Column::DefaultBattleStyle",
        to = "super::battle_types::Column::Id"
    )]
    BattleType,
}

impl Related<super::quest_aliases::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::QuestAlias.def()
    }
}

impl Related<super::battle_types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleType.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
