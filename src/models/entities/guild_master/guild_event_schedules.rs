use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "guild_event_schedules")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub event_type: String,
    pub event_count: i64,
    pub profile: String,
    pub weak_attribute: i32,
    pub start_at: DateTime, // スプレッドシート（JST）と一致させる - timestamp型（タイムゾーンなし）
    pub end_at: DateTime,   // スプレッドシート（JST）と一致させる - timestamp型（タイムゾーンなし）
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
            guild_id: sea_orm::NotSet,
            id: sea_orm::NotSet,
            event_type: sea_orm::NotSet,
            event_count: sea_orm::NotSet,
            profile: sea_orm::NotSet,
            weak_attribute: sea_orm::NotSet,
            start_at: sea_orm::NotSet,
            end_at: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
