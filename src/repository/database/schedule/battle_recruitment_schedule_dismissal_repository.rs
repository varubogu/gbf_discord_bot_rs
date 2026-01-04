use crate::models::entities::guild_master::battle_recruitment_schedule_dismissals;
use crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository;
use crate::types::{AppError, Result};
use async_trait::async_trait;
use sea_orm::entity::prelude::TimeTime;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

/// 定期募集解散リポジトリ
#[derive(Default)]
pub struct SeaOrmBattleRecruitmentScheduleDismissalRepository;

#[async_trait]
impl BattleRecruitmentScheduleDismissalRepository
    for SeaOrmBattleRecruitmentScheduleDismissalRepository
{
    /// 解散時刻を作成（絶対時刻）
    async fn create_absolute(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
        input_value: String,
        dismissal_time: TimeTime,
    ) -> Result<battle_recruitment_schedule_dismissals::Model> {
        debug!(
            schedule_id,
            input_value,
            dismissal_time = %dismissal_time,
            "定期募集の解散時刻を作成します（絶対時刻）"
        );

        let active_model = battle_recruitment_schedule_dismissals::ActiveModel {
            schedule_id: Set(schedule_id),
            input_value: Set(input_value),
            input_type: Set(1), // Absolute
            dismissal_time: Set(Some(dismissal_time)),
            relative_days: Set(None),
            relative_hours: Set(None),
            relative_minutes: Set(None),
            ..Default::default()
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, schedule_id, "定期募集の解散時刻の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            id = model.id,
            schedule_id, "定期募集の解散時刻を作成しました（絶対時刻）"
        );
        Ok(model)
    }

    /// 解散時刻を作成（相対時刻）
    async fn create_relative(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
        input_value: String,
        relative_days: i32,
        relative_hours: i32,
        relative_minutes: i32,
    ) -> Result<battle_recruitment_schedule_dismissals::Model> {
        debug!(
            schedule_id,
            input_value,
            relative_days,
            relative_hours,
            relative_minutes,
            "定期募集の解散時刻を作成します（相対時刻）"
        );

        let active_model = battle_recruitment_schedule_dismissals::ActiveModel {
            schedule_id: Set(schedule_id),
            input_value: Set(input_value),
            input_type: Set(2), // Relative
            dismissal_time: Set(None),
            relative_days: Set(Some(relative_days)),
            relative_hours: Set(Some(relative_hours)),
            relative_minutes: Set(Some(relative_minutes)),
            ..Default::default()
        };

        let model = active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, schedule_id, "定期募集の解散時刻の作成に失敗しました");
            AppError::Database(e)
        })?;

        debug!(
            id = model.id,
            schedule_id, "定期募集の解散時刻を作成しました（相対時刻）"
        );
        Ok(model)
    }

    /// schedule_idで解散時刻を取得
    async fn find_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<Vec<battle_recruitment_schedule_dismissals::Model>> {
        debug!(schedule_id, "定期募集の解散時刻を取得します");

        let models = battle_recruitment_schedule_dismissals::Entity::find()
            .filter(battle_recruitment_schedule_dismissals::Column::ScheduleId.eq(schedule_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, schedule_id, "定期募集の解散時刻の取得に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            schedule_id,
            count = models.len(),
            "定期募集の解散時刻を取得しました"
        );
        Ok(models)
    }

    /// schedule_idで解散時刻を削除
    async fn delete_by_schedule_id(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<u64> {
        debug!(schedule_id, "定期募集の解散時刻を削除します");

        let result = battle_recruitment_schedule_dismissals::Entity::delete_many()
            .filter(battle_recruitment_schedule_dismissals::Column::ScheduleId.eq(schedule_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, schedule_id, "定期募集の解散時刻の削除に失敗しました");
                AppError::Database(e)
            })?;

        debug!(
            schedule_id,
            deleted_count = result.rows_affected,
            "定期募集の解散時刻を削除しました"
        );
        Ok(result.rows_affected)
    }
}

impl SeaOrmBattleRecruitmentScheduleDismissalRepository {
    pub fn new() -> Self {
        Self
    }
}
