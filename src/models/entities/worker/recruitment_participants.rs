use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "recruitment_participants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub recruitment_id: i32,
    pub user_id: i64,
    pub element_id: Option<i32>,
    pub participated_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::battle_recruitments::Entity",
        from = "Column::RecruitmentId",
        to = "super::battle_recruitments::Column::Id"
    )]
    BattleRecruitment,
    #[sea_orm(
        belongs_to = "super::super::master::elements::Entity",
        from = "Column::ElementId",
        to = "super::super::master::elements::Column::Id"
    )]
    Element,
}

impl Related<super::battle_recruitments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BattleRecruitment.def()
    }
}

impl Related<super::super::master::elements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Element.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: sea_orm::NotSet,
            recruitment_id: sea_orm::NotSet,
            user_id: sea_orm::NotSet,
            element_id: sea_orm::NotSet,
            participated_at: sea_orm::Set(now),
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}
