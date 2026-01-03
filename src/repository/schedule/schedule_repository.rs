use crate::models::entities::master::{event_schedule_details, event_schedules};
use crate::types::Result;
use async_trait::async_trait;

/// スケジュールリポジトリの抽象インターフェース
#[async_trait]
pub trait ScheduleRepository: Send + Sync {
    /// すべてのイベントスケジュール詳細を取得
    async fn find_all_event_schedule_details<C>(
        &self,
        db: &C,
    ) -> Result<Vec<event_schedule_details::Model>>
    where
        C: sea_orm::ConnectionTrait;

    /// すべてのイベントスケジュールを取得
    async fn find_all_event_schedules<C>(
        &self,
        db: &C,
    ) -> Result<Vec<event_schedules::Model>>
    where
        C: sea_orm::ConnectionTrait;
}
