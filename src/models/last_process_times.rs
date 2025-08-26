use crate::models::entities::{
    last_process_times, last_process_times::Entity as LastProcessTimeEntity,
    last_process_times::LastProcessType,
};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastProcessTime {
    pub process_type: i32,
    pub execute_time: Option<chrono::DateTime<chrono::Utc>>,
    pub memo: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<last_process_times::Model> for LastProcessTime {
    fn from(model: last_process_times::Model) -> Self {
        Self {
            process_type: model.process_type,
            execute_time: model.execute_time,
            memo: model.memo,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl LastProcessTime {
    pub fn get_process_type_enum(&self) -> Option<LastProcessType> {
        LastProcessType::from_i32(self.process_type)
    }

    pub fn is_process_type(&self, process_type: LastProcessType) -> bool {
        self.process_type == process_type.as_i32()
    }
}

impl Database {
    pub async fn get_last_process_times(&self) -> Result<Vec<LastProcessTime>, DbErr> {
        let models = LastProcessTimeEntity::find().all(&self.conn).await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_last_process_time_by_type(
        &self,
        process_type: LastProcessType,
    ) -> Result<Option<LastProcessTime>, DbErr> {
        let last_process_time = LastProcessTimeEntity::find()
            .filter(last_process_times::Column::ProcessType.eq(process_type.as_i32()))
            .one(&self.conn)
            .await?;

        Ok(last_process_time.map(|lpt| lpt.into()))
    }

    pub async fn get_last_process_time_by_id(
        &self,
        process_type: i32,
    ) -> Result<Option<LastProcessTime>, DbErr> {
        let last_process_time = LastProcessTimeEntity::find_by_id(process_type)
            .one(&self.conn)
            .await?;

        Ok(last_process_time.map(|lpt| lpt.into()))
    }

    pub async fn get_schedule_last_process_time(&self) -> Result<Option<LastProcessTime>, DbErr> {
        self.get_last_process_time_by_type(LastProcessType::Schedule)
            .await
    }

    pub async fn get_spreadsheet_load_last_process_time(
        &self,
    ) -> Result<Option<LastProcessTime>, DbErr> {
        self.get_last_process_time_by_type(LastProcessType::SpreadsheetLoad)
            .await
    }

    pub async fn get_spreadsheet_push_last_process_time(
        &self,
    ) -> Result<Option<LastProcessTime>, DbErr> {
        self.get_last_process_time_by_type(LastProcessType::SpreadsheetPush)
            .await
    }
}
