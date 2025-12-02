use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "master", table_name = "quests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub default_battle_style_id: i32,
    pub recruit_count: i32,
    pub available_battle_style_ids: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::quest_aliases::Entity")]
    QuestAlias,
    #[sea_orm(
        belongs_to = "super::battle_styles::Entity",
        from = "Column::DefaultBattleStyleId",
        to = "super::battle_styles::Column::Id"
    )]
    BattleStyle,
}

impl Related<super::quest_aliases::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::QuestAlias.def()
    }
}

impl Related<super::battle_styles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleStyle.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
