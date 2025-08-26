use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "guilds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub discord_guild_id: i64,
    pub guild_name: String,
    pub recruit_channel_id: Option<i64>,
    pub notification_channel_id: Option<i64>,
    pub timezone: Option<String>,
    pub default_recruit_duration: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::battle_recruitments::Entity")]
    BattleRecruitment,
    #[sea_orm(has_many = "super::message_texts::Entity")]
    MessageText,
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl Related<super::message_texts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageText.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
