//! 自動募集設定エンティティ
//!
//! ギルドごとの自動募集機能の設定を管理する

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "guild_master", table_name = "auto_recruitments")]
pub struct Model {
    /// ギルドID（主キー）
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    /// カテゴリチャンネルID
    pub category_id: i64,
    /// マッチング済みチャンネルID
    pub matching_channel_id: Option<i64>,
    /// クエストチャンネルID
    pub quest_channel_id: Option<i64>,
    /// マッチングチャンネルがBot作成かどうか
    pub matching_channel_is_bot_created: bool,
    /// クエストチャンネルがBot作成かどうか
    pub quest_channel_is_bot_created: bool,
    /// マッチングチャンネルに送信したメッセージID
    pub matching_message_id: Option<i64>,
    /// クエストチャンネルに送信したメッセージID
    pub quest_message_id: Option<i64>,
    /// 募集日数（2-7日、デフォルト7日）
    pub days_range: i32,
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
            category_id: sea_orm::NotSet,
            matching_channel_id: sea_orm::NotSet,
            quest_channel_id: sea_orm::NotSet,
            matching_channel_is_bot_created: sea_orm::Set(false),
            quest_channel_is_bot_created: sea_orm::Set(false),
            matching_message_id: sea_orm::NotSet,
            quest_message_id: sea_orm::NotSet,
            days_range: sea_orm::Set(7), // デフォルト7日
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
