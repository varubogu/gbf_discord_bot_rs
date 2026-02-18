use crate::models::entities::worker::notification_rel_battle_recruitments::{
    self, Entity as NotificationRelBattleRecruitmentEntity,
};
use crate::infrastructure::database::repositories::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRelBattleRecruitment {
    pub recruit_id: i32,
    pub notification_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<notification_rel_battle_recruitments::Model> for NotificationRelBattleRecruitment {
    fn from(model: notification_rel_battle_recruitments::Model) -> Self {
        Self {
            recruit_id: model.recruit_id,
            notification_id: model.notification_id,
            created_at: model.created_at,
        }
    }
}

impl Database {
    pub async fn get_notification_rel_battle_recruitments(
        &self,
    ) -> Result<Vec<NotificationRelBattleRecruitment>, DbErr> {
        let models = NotificationRelBattleRecruitmentEntity::find()
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_rel_battle_recruitment_by_ids(
        &self,
        recruit_id: i32,
        notification_id: i32,
    ) -> Result<Option<NotificationRelBattleRecruitment>, DbErr> {
        let relation = NotificationRelBattleRecruitmentEntity::find()
            .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruit_id))
            .filter(
                notification_rel_battle_recruitments::Column::NotificationId.eq(notification_id),
            )
            .one(&self.conn)
            .await?;

        Ok(relation.map(|r| r.into()))
    }

    pub async fn get_notification_rel_battle_recruitments_by_recruit(
        &self,
        recruit_id: i32,
    ) -> Result<Vec<NotificationRelBattleRecruitment>, DbErr> {
        let models = NotificationRelBattleRecruitmentEntity::find()
            .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruit_id))
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_notification_rel_battle_recruitments_by_notification(
        &self,
        notification_id: i32,
    ) -> Result<Vec<NotificationRelBattleRecruitment>, DbErr> {
        let models = NotificationRelBattleRecruitmentEntity::find()
            .filter(
                notification_rel_battle_recruitments::Column::NotificationId.eq(notification_id),
            )
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
