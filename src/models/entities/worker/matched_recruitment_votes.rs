//! マッチング投票エンティティ
//!
//! 自動募集でマッチング成立後のクエスト投票を管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "matched_recruitment_votes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// マッチング済みチャンネルID
    pub matched_channel_id: i32,
    /// 投票したユーザーID
    pub user_id: i64,
    /// 投票したクエストID（NULLなら「何でも良い」）
    pub quest_id: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::matched_recruitment_channels::Entity",
        from = "Column::MatchedChannelId",
        to = "super::matched_recruitment_channels::Column::Id"
    )]
    MatchedRecruitmentChannel,
}

impl Related<super::matched_recruitment_channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchedRecruitmentChannel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            matched_channel_id: sea_orm::NotSet,
            user_id: sea_orm::NotSet,
            quest_id: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
