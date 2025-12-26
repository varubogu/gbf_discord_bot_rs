use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "guild_last_process_times")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub process_type: i32,
    pub execute_time: Option<DateTimeUtc>,
    pub memo: String,
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
}

impl Related<super::super::guild_master::guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guild.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            guild_id: sea_orm::NotSet,
            process_type: sea_orm::NotSet,
            execute_time: sea_orm::NotSet,
            memo: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}

// 処理種類のenum定義はlast_process_timesと共通のため、
// 必要に応じてsuper::last_process_times::LastProcessTypeを使用
