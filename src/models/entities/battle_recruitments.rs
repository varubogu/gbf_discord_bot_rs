use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "battle_recruitments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_at: DateTimeUtc,
    pub is_recruiting: bool,
    pub is_canceled: bool,
    pub recruit_end_message_id: Option<i64>,
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
    #[sea_orm(
        belongs_to = "super::battle_types::Entity",
        from = "Column::BattleStyleId",
        to = "super::battle_types::Column::Id"
    )]
    BattleType,
    #[sea_orm(
        belongs_to = "super::guilds::Entity",
        from = "Column::GuildId",
        to = "super::guilds::Column::GuildId"
    )]
    Guild,
}

impl Related<super::quests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quest.def()
    }
}

impl Related<super::battle_types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleType.def()
    }
}

impl Related<super::guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guild.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            guild_id: sea_orm::NotSet,
            channel_id: sea_orm::NotSet,
            message_id: sea_orm::NotSet,
            quest_id: sea_orm::NotSet,
            battle_style_id: sea_orm::NotSet,
            quest_start_at: sea_orm::NotSet,
            is_recruiting: sea_orm::Set(true),
            is_canceled: sea_orm::Set(false),
            recruit_end_message_id: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
