//! 自動募集日時チャンネルエンティティ
//!
//! 自動募集機能で使用する日時チャンネルを管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "auto_recruitment_channels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    /// チャンネルID
    pub channel_id: i64,
    /// 何月の募集か
    pub month: i32,
    /// 何日の募集か
    pub day: i32,
    /// 並び順
    pub sort_order: i32,
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
            id: sea_orm::NotSet,
            guild_id: sea_orm::NotSet,
            channel_id: sea_orm::NotSet,
            month: sea_orm::NotSet,
            day: sea_orm::NotSet,
            sort_order: sea_orm::Set(0),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
