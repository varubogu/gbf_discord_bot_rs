use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "battle_recruitment_schedules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub guild_id: i64,
    pub channel_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_time: TimeTime,
    pub recruit_start_day_offset: i32,
    pub recruit_start_time: Option<TimeTime>,
    pub max_participants: Option<i32>,
    pub note: Option<String>,
    pub is_enabled: bool,
    pub created_by: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::guild_master::guilds::Entity",
        from = "Column::GuildId",
        to = "super::super::guild_master::guilds::Column::GuildId"
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
        from = "Column::BattleStyleId",
        to = "super::super::master::battle_styles::Column::Id"
    )]
    BattleStyle,
    #[sea_orm(has_many = "super::super::guild_master::battle_recruitment_schedule_days::Entity")]
    BattleRecruitmentScheduleDays,
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
        Relation::BattleStyle.def()
    }
}

impl Related<super::battle_recruitment_schedule_days::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitmentScheduleDays.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            name: sea_orm::NotSet,
            guild_id: sea_orm::NotSet,
            channel_id: sea_orm::NotSet,
            quest_id: sea_orm::NotSet,
            battle_style_id: sea_orm::NotSet,
            quest_start_time: sea_orm::NotSet,
            recruit_start_day_offset: sea_orm::NotSet, // デフォルト値はコマンド層で決定
            recruit_start_time: sea_orm::NotSet,
            max_participants: sea_orm::NotSet,
            note: sea_orm::NotSet,
            is_enabled: sea_orm::Set(true),
            created_by: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
