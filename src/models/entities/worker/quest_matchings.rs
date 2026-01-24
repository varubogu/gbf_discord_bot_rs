//! マッチングエンティティ
//!
//! 自動募集でマッチング成立したグループを管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "quest_matchings")]
pub struct Model {
    /// ギルドID（複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    /// マッチングID（UUID、複合主キーの一部）
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// クエストID
    pub quest_id: i32,
    /// 予定月
    pub scheduled_month: i32,
    /// 予定日
    pub scheduled_day: i32,
    /// 予定時間（5-28）
    pub scheduled_hour: i32,
    /// 状態: active, completed, cancelled
    pub status: String,
    /// 作成されたマルチ募集ID（募集作成後にセット）
    pub recruitment_id: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::master::quests::Entity",
        from = "Column::QuestId",
        to = "super::super::master::quests::Column::Id"
    )]
    Quest,
    #[sea_orm(has_many = "super::quest_matching_users::Entity")]
    MatchingUsers,
}

impl Related<super::super::master::quests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quest.def()
    }
}

impl Related<super::quest_matching_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchingUsers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            guild_id: sea_orm::NotSet,
            id: sea_orm::Set(Uuid::new_v4()),
            quest_id: sea_orm::NotSet,
            scheduled_month: sea_orm::NotSet,
            scheduled_day: sea_orm::NotSet,
            scheduled_hour: sea_orm::NotSet,
            status: sea_orm::Set("active".to_string()),
            recruitment_id: sea_orm::Set(None),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
