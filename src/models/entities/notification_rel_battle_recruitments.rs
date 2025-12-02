use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "notification_rel_battle_recruitments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub recruit_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub notification_id: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::notifications::Entity",
        from = "Column::NotificationId",
        to = "super::notifications::Column::Id"
    )]
    Notification,
    #[sea_orm(
        belongs_to = "super::battle_recruitments::Entity",
        from = "Column::RecruitId",
        to = "super::battle_recruitments::Column::Id"
    )]
    BattleRecruitment,
}

impl Related<super::notifications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notification.def()
    }
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            recruit_id: sea_orm::NotSet,
            notification_id: sea_orm::NotSet,
            created_at: sea_orm::Set(chrono::Utc::now()),
        }
    }
}
