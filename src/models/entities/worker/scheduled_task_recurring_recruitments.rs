use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(
    schema_name = "worker",
    table_name = "scheduled_task_recurring_recruitments"
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub scheduled_task_id: i32,
    pub recruitment_schedule_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // NOTE: belongs_toリレーションは相互参照の問題を避けるため、
    // 必要に応じて手動でJOINクエリを実装する
}

impl ActiveModelBehavior for ActiveModel {}
