use crate::models::entities::{
    last_process_times, last_process_times::Entity as LastProcessTimeEntity,
    last_process_times::LastProcessType,
};
use crate::models::last_process_times::LastProcessTime;
use crate::types::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// last_process_timesリポジトリ
pub struct LastProcessTimeRepository;

impl LastProcessTimeRepository {
    pub fn new() -> Self {
        Self
    }

    // /// すべてのlast_process_timesを取得
    // pub async fn find_all(&self) -> Result<Vec<LastProcessTime>> {
    //     let models = LastProcessTimeEntity::find().all(&self.db).await?;
    //     Ok(models.into_iter().map(|model| model.into()).collect())
    // }

    /// process_typeでlast_process_timeを取得
    pub async fn find_by_type<'c, C>(
        &self,
        db: &'c C,
        process_type: LastProcessType,
    ) -> Result<Option<LastProcessTime>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let last_process_time = LastProcessTimeEntity::find()
            .filter(last_process_times::Column::ProcessType.eq(process_type.as_i32()))
            .one(db)
            .await?;

        Ok(last_process_time.map(|lpt| lpt.into()))
    }

    // /// IDでlast_process_timeを取得
    // pub async fn find_by_id(&self, process_type: i32) -> Result<Option<LastProcessTime>> {
    //     let last_process_time = LastProcessTimeEntity::find_by_id(process_type)
    //         .one(&self.db)
    //         .await?;

    //     Ok(last_process_time.map(|lpt| lpt.into()))
    // }

    /// スケジュール処理のlast_process_timeを取得
    pub async fn find_schedule_last_process_time<'c, C>(
        &self,
        db: &'c C,
    ) -> Result<Option<LastProcessTime>>
    where
        C: sea_orm::ConnectionTrait,
    {
        self.find_by_type(db, LastProcessType::Schedule).await
    }

    // /// スプレッドシート読み込みのlast_process_timeを取得
    // pub async fn find_spreadsheet_load_last_process_time(
    //     &self,
    // ) -> Result<Option<LastProcessTime>> {
    //     self.find_by_type(LastProcessType::SpreadsheetLoad).await
    // }

    // /// スプレッドシート書き込みのlast_process_timeを取得
    // pub async fn find_spreadsheet_push_last_process_time(
    //     &self,
    // ) -> Result<Option<LastProcessTime>> {
    //     self.find_by_type(LastProcessType::SpreadsheetPush).await
    // }

    /// last_process_timeを更新（トランザクション付き）
    /// レコードが存在しない場合は新規作成、存在する場合は更新
    pub async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        process_type: LastProcessType,
        execute_time: DateTime<Utc>,
    ) -> Result<last_process_times::Model> {
        let now = Utc::now();

        debug!(
            process_type = ?process_type,
            execute_time = %execute_time,
            "last_process_timeをupsertします"
        );

        // 既存のレコードを検索
        let existing = LastProcessTimeEntity::find_by_id(process_type.as_i32())
            .one(txn)
            .await?;

        let model = if let Some(existing_model) = existing {
            // 更新
            let mut active_model: last_process_times::ActiveModel = existing_model.into();
            active_model.execute_time = Set(Some(execute_time));
            active_model.updated_at = Set(now);

            active_model.update(txn).await.map_err(|e| {
                error!(error = %e, "last_process_timeの更新に失敗しました");
                e
            })?
        } else {
            // 新規作成
            let active_model = last_process_times::ActiveModel {
                process_type: Set(process_type.as_i32()),
                execute_time: Set(Some(execute_time)),
                memo: Set(process_type.memo().to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active_model.insert(txn).await.map_err(|e| {
                error!(error = %e, "last_process_timeの作成に失敗しました");
                e
            })?
        };

        debug!(
            process_type = ?process_type,
            "last_process_timeをupsertしました"
        );
        Ok(model)
    }
}
