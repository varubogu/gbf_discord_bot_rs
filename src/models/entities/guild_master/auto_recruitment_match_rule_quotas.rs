use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "guild_master",
    table_name = "auto_recruitment_match_rule_quotas"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub battle_style_id: i32,
    pub required_count: i32,
    pub sort_order: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::auto_recruitment_match_rules::Entity",
        from = "(Column::GuildId, Column::QuestId)",
        to = "(super::auto_recruitment_match_rules::Column::GuildId, super::auto_recruitment_match_rules::Column::QuestId)"
    )]
    Rule,
    #[sea_orm(
        belongs_to = "super::super::master::battle_styles::Entity",
        from = "Column::BattleStyleId",
        to = "super::super::master::battle_styles::Column::Id"
    )]
    BattleStyle,
}

impl Related<super::auto_recruitment_match_rules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}

impl Related<super::super::master::battle_styles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleStyle.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
