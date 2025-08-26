use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub schedule_datetime: DateTimeUtc,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_text_id: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // NOTE: belongs_toリレーションは相互参照の問題を避けるため、
    // 必要に応じて手動でJOINクエリを実装する
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            schedule_datetime: sea_orm::NotSet,
            guild_id: sea_orm::NotSet,
            channel_id: sea_orm::NotSet,
            message_text_id: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
