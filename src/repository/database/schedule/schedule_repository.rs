use crate::models::entities::{event_schedule_details, event_schedules};
use crate::types::Result;
use sea_orm::{DatabaseConnection, EntityTrait};
use tracing::{debug, error};

/// スケジュールリポジトリ
pub struct ScheduleRepository {
    db: DatabaseConnection,
}

impl ScheduleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // /// 現在有効なイベントスケジュールを取得
    // pub async fn find_active_event_schedules(
    //     &self,
    //     now: DateTime<Utc>,
    // ) -> Result<Vec<event_schedules::Model>> {
    //     debug!(now = %now, "有効なイベントスケジュールを取得します");

    //     let schedules = event_schedules::Entity::find()
    //         .filter(event_schedules::Column::StartAt.lte(now))
    //         .filter(event_schedules::Column::EndAt.gte(now))
    //         .all(&self.db)
    //         .await
    //         .map_err(|e| {
    //             error!(error = %e, "イベントスケジュールの取得に失敗しました");
    //             e
    //         })?;

    //     debug!(count = schedules.len(), "イベントスケジュールを取得しました");
    //     Ok(schedules)
    // }

    // /// プロファイルでイベントスケジュールを取得
    // pub async fn find_event_schedules_by_profile(
    //     &self,
    //     profile: &str,
    // ) -> Result<Vec<event_schedules::Model>> {
    //     debug!(profile = %profile, "プロファイルでイベントスケジュールを取得します");

    //     let schedules = event_schedules::Entity::find()
    //         .filter(event_schedules::Column::Profile.eq(profile))
    //         .all(&self.db)
    //         .await
    //         .map_err(|e| {
    //             error!(error = %e, "イベントスケジュールの取得に失敗しました");
    //             e
    //         })?;

    //     debug!(count = schedules.len(), "イベントスケジュールを取得しました");
    //     Ok(schedules)
    // }

    /// すべてのイベントスケジュール詳細を取得
    pub async fn find_all_event_schedule_details(
        &self,
    ) -> Result<Vec<event_schedule_details::Model>> {
        debug!("すべてのイベントスケジュール詳細を取得します");

        let details = event_schedule_details::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!(error = %e, "イベントスケジュール詳細の取得に失敗しました");
                e
            })?;

        debug!(count = details.len(), "イベントスケジュール詳細を取得しました");
        Ok(details)
    }

    // /// プロファイルでイベントスケジュール詳細を取得
    // pub async fn find_event_schedule_details_by_profile(
    //     &self,
    //     profile: &str,
    // ) -> Result<Vec<event_schedule_details::Model>> {
    //     debug!(profile = %profile, "プロファイルでイベントスケジュール詳細を取得します");

    //     let details = event_schedule_details::Entity::find()
    //         .filter(event_schedule_details::Column::Profile.eq(profile))
    //         .all(&self.db)
    //         .await
    //         .map_err(|e| {
    //             error!(error = %e, "イベントスケジュール詳細の取得に失敗しました");
    //             e
    //         })?;

    //     debug!(count = details.len(), "イベントスケジュール詳細を取得しました");
    //     Ok(details)
    // }

    /// すべてのイベントスケジュールを取得
    pub async fn find_all_event_schedules(&self) -> Result<Vec<event_schedules::Model>> {
        debug!("すべてのイベントスケジュールを取得します");

        let schedules = event_schedules::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!(error = %e, "イベントスケジュールの取得に失敗しました");
                e
            })?;

        debug!(count = schedules.len(), "イベントスケジュールを取得しました");
        Ok(schedules)
    }
}
