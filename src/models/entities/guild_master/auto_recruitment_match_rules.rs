use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "guild_master",
    table_name = "auto_recruitment_match_rules"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    pub preset_type: String,
    pub min_match_count: i32,
    pub required_battle_style_id: Option<i32>,
    pub required_battle_style_count: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::guilds::Entity",
        from = "Column::GuildId",
        to = "super::guilds::Column::GuildId"
    )]
    Guild,
    #[sea_orm(
        belongs_to = "super::super::master::quests::Entity",
        from = "Column::QuestId",
        to = "super::super::master::quests::Column::Id"
    )]
    Quest,
    #[sea_orm(
        belongs_to = "super::super::master::battle_styles::Entity",
        from = "Column::RequiredBattleStyleId",
        to = "super::super::master::battle_styles::Column::Id"
    )]
    RequiredBattleStyle,
    #[sea_orm(has_many = "super::auto_recruitment_match_rule_quotas::Entity")]
    Quotas,
}

impl Related<super::guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guild.def()
    }
}

impl Related<super::super::master::quests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quest.def()
    }
}

impl Related<super::super::master::battle_styles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RequiredBattleStyle.def()
    }
}

impl Related<super::auto_recruitment_match_rule_quotas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quotas.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
