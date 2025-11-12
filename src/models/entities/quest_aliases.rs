use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quest_aliases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sequence_no: i32,
    pub alias: String,
    pub alias_kana_small: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::quests::Entity",
        from = "Column::QuestId",
        to = "super::quests::Column::Id"
    )]
    Quest,
}

impl Related<super::quests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
