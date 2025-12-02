use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "master", table_name = "event_schedule_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub profile: String,
    pub start_day_relative: String,
    pub time: String,
    pub schedule_name: String,
    pub message_text_id: String,
    pub notification_channel_type: i32,
    pub reactions: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // profileカラムでevent_schedulesテーブルと関連付け
    // NOTE: Sea-ORMでは文字列による関連付けは直接サポートされていないため、
    // 必要に応じて手動でJOINクエリを実装する
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            profile: sea_orm::NotSet,
            start_day_relative: sea_orm::NotSet,
            time: sea_orm::NotSet,
            schedule_name: sea_orm::NotSet,
            message_text_id: sea_orm::NotSet,
            notification_channel_type: sea_orm::NotSet,
            reactions: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
