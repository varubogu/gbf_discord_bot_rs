use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "elements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub reaction_stamp: Option<String>,
    pub name_jp: String,
    pub name_en: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
