//! マッチングユーザーエンティティ
//!
//! マッチングに参加しているユーザーを管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "quest_matching_users")]
pub struct Model {
    /// ギルドID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    /// マッチングID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub matching_id: Uuid,
    /// ユーザーID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    /// 担当属性（6属性クエストの場合のみ、NULLなら未決定）
    pub battle_style_id: Option<i32>,
    /// 参加日時
    pub joined_at: DateTimeUtc,
    /// 離脱日時（NULLなら参加中）
    pub left_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::quest_matchings::Entity",
        from = "(Column::GuildId, Column::MatchingId)",
        to = "(super::quest_matchings::Column::GuildId, super::quest_matchings::Column::Id)"
    )]
    Matching,
}

impl Related<super::quest_matchings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Matching.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            guild_id: sea_orm::NotSet,
            matching_id: sea_orm::NotSet,
            user_id: sea_orm::NotSet,
            battle_style_id: sea_orm::Set(None),
            joined_at: sea_orm::Set(now),
            left_at: sea_orm::Set(None),
        }
    }
}
