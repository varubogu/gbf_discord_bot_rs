//! ユーザー希望クエストエンティティ
//!
//! ユーザーが自動募集で希望するクエストを管理する
//! 6属性クエストの場合はbattle_style_idで希望属性も保存する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "user_desired_quests")]
pub struct Model {
    /// ギルドID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    /// ユーザーID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    /// クエストID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    /// 希望属性（複合主キーの一部）
    /// 0: 属性指定なしクエスト、1-6: 各属性
    #[sea_orm(primary_key, auto_increment = false)]
    pub battle_style_id: i32,
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

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            guild_id: sea_orm::NotSet,
            user_id: sea_orm::NotSet,
            quest_id: sea_orm::NotSet,
            battle_style_id: sea_orm::Set(0),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
