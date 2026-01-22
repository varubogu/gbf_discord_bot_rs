//! マッチング済み募集チャンネルエンティティ
//!
//! 自動募集でマッチングが成立した募集を管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "matched_recruitment_channels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    /// マッチングチャンネルID
    pub channel_id: i64,
    /// 通知メッセージID
    pub message_id: i64,
    /// 何月の募集か
    pub month: i32,
    /// 何日の募集か
    pub day: i32,
    /// 何時の募集か
    pub hour: i32,
    /// 決定したクエストID（投票完了後にセット）
    pub quest_id: Option<i32>,
    /// クエストが決定済みかどうか
    pub is_decided: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::matched_recruitment_votes::Entity")]
    MatchedRecruitmentVotes,
}

impl Related<super::matched_recruitment_votes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchedRecruitmentVotes.def()
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
            month: sea_orm::NotSet,
            day: sea_orm::NotSet,
            hour: sea_orm::NotSet,
            quest_id: sea_orm::NotSet,
            is_decided: sea_orm::Set(false),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
