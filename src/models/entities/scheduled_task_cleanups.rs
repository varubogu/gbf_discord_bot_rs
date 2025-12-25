use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "scheduled_task_cleanups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    pub target_schema: String,
    pub target_table: String,
    pub cleanup_before: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // NOTE: belongs_toリレーションは相互参照の問題を避けるため、
    // 必要に応じて手動でJOINクエリを実装する
}

impl ActiveModelBehavior for ActiveModel {}
