use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "guilds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    pub name: String,
    pub recruit_channel_id: Option<i64>,
    pub timezone: Option<String>,
    pub default_recruit_duration: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::battle_recruitments::Entity")]
    BattleRecruitment,
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
