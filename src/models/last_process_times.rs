use crate::models::entities::worker::last_process_times::{
    self, Entity as LastProcessTimeEntity, LastProcessType,
};
use crate::infrastructure::database::repositories::db_compat::Database;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, Set,
};
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

    /// last_process_timeを更新（トランザクション付き）
    /// レコードが存在しない場合は新規作成、存在する場合は更新
    pub async fn upsert_last_process_time_with_txn(
        &self,
        txn: &DatabaseTransaction,
        process_type: LastProcessType,
        execute_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<last_process_times::Model, DbErr> {
        let now = chrono::Utc::now();

        // 既存のレコードを検索
        let existing = LastProcessTimeEntity::find_by_id(process_type.as_i32())
            .one(txn)
            .await?;

        if let Some(existing_model) = existing {
            // 更新
            let mut active_model: last_process_times::ActiveModel = existing_model.into();
            active_model.execute_time = Set(Some(execute_time));
            active_model.updated_at = Set(now);

            active_model.update(txn).await
        } else {
            // 新規作成
            let active_model = last_process_times::ActiveModel {
                process_type: Set(process_type.as_i32()),
                execute_time: Set(Some(execute_time)),
                memo: Set(process_type.memo().to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active_model.insert(txn).await
        }
    }
}
