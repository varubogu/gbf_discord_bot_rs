//! 自動募集参加可能時間エンティティ
//!
//! ユーザーが自動募集で参加可能な日時を管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "guild_master",
    table_name = "auto_recruitment_participants"
)]
pub struct Model {
    /// ギルドID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    /// ユーザーID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    /// 何月の募集か（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub month: i32,
    /// 何日の募集か（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub day: i32,
    /// 何時の募集か（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub hour: i32,
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
            user_id: sea_orm::NotSet,
            month: sea_orm::NotSet,
            day: sea_orm::NotSet,
            hour: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
