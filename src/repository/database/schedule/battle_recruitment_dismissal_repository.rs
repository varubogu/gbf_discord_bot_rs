use crate::models::entities::worker::battle_recruitment_dismissals;
use crate::types::{AppError, Result};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// マルチ募集解散リポジトリ
#[derive(Default)]
pub struct SeaOrmBattleRecruitmentDismissalRepository;

impl SeaOrmBattleRecruitmentDismissalRepository {
    pub fn new() -> Self {
        Self
    }

    /// 解散時刻を作成（絶対日時）
    pub async fn create_absolute(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        input_value: String,
        dismissal_datetime: DateTime<Utc>,
    ) -> Result<battle_recruitment_dismissals::Model> {
        debug!(
            recruitment_id,
            input_value,
            dismissal_datetime = %dismissal_datetime,
            "解散時刻を作成します（絶対日時）"
        );

        let active_model = battle_recruitment_dismissals::ActiveModel {
            recruitment_id: Set(recruitment_id),
            input_value: Set(input_value),
            input_type: Set(1), // Absolute
            dismissal_datetime: Set(Some(dismissal_datetime)),
            relative_days: Set(None),
            relative_hours: Set(None),
            relative_minutes: Set(None),
            ..Default::default()
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, recruitment_id, "解散時刻の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            id = model.id,
            recruitment_id, "解散時刻を作成しました（絶対日時）"
        );
        Ok(model)
    }

    /// 解散時刻を作成（相対時刻）
    pub async fn create_relative(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        input_value: String,
        relative_days: i32,
        relative_hours: i32,
        relative_minutes: i32,
    ) -> Result<battle_recruitment_dismissals::Model> {
        debug!(
            recruitment_id,
            input_value,
            relative_days,
            relative_hours,
            relative_minutes,
            "解散時刻を作成します（相対時刻）"
        );

        let active_model = battle_recruitment_dismissals::ActiveModel {
            recruitment_id: Set(recruitment_id),
            input_value: Set(input_value),
            input_type: Set(2), // Relative
            dismissal_datetime: Set(None),
            relative_days: Set(Some(relative_days)),
            relative_hours: Set(Some(relative_hours)),
            relative_minutes: Set(Some(relative_minutes)),
            ..Default::default()
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, recruitment_id, "解散時刻の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            id = model.id,
            recruitment_id, "解散時刻を作成しました（相対時刻）"
        );
        Ok(model)
    }

    /// recruitment_idで解散時刻を取得
    pub async fn find_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<battle_recruitment_dismissals::Model>> {
        debug!(recruitment_id, "解散時刻を取得します");

        let models = battle_recruitment_dismissals::Entity::find()
            .filter(battle_recruitment_dismissals::Column::RecruitmentId.eq(recruitment_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_id, "解散時刻の取得に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            recruitment_id,
            count = models.len(),
            "解散時刻を取得しました"
        );
        Ok(models)
    }

    /// idで解散時刻を取得
    pub async fn find_by_id(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
    ) -> Result<Option<battle_recruitment_dismissals::Model>> {
        debug!(id, "解散時刻をIDで取得します");

        let model = battle_recruitment_dismissals::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id, "解散時刻の取得に失敗しました");
                AppError::Database(e)
            })?;

        debug!(id, found = model.is_some(), "解散時刻をIDで取得しました");
        Ok(model)
    }

    /// recruitment_idで解散時刻を削除
    pub async fn delete_by_recruitment_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<u64> {
        debug!(recruitment_id, "解散時刻を削除します");

        let result = battle_recruitment_dismissals::Entity::delete_many()
            .filter(battle_recruitment_dismissals::Column::RecruitmentId.eq(recruitment_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, recruitment_id, "解散時刻の削除に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            recruitment_id,
            deleted_count = result.rows_affected,
            "解散時刻を削除しました"
        );
        Ok(result.rows_affected)
    }
}
