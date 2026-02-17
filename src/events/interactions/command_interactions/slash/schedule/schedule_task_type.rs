use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use poise::ChoiceParameter;

/// スケジュール再生成コマンドで指定可能なタスク種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, ChoiceParameter)]
pub enum ScheduleTaskTypeChoice {
    #[name = "通知"]
    Notification,
    #[name = "解散"]
    Dissolution,
    #[name = "データクリーンアップ"]
    DataCleanup,
    #[name = "定期募集実行"]
    RecurringRecruitment,
    #[name = "人数不足解散"]
    Dismissal,
    #[name = "自動募集日付ローテーション"]
    AutoRecruitmentRotation,
    #[name = "自動マッチング"]
    AutoMatching,
}

impl From<ScheduleTaskTypeChoice> for ScheduledTaskType {
    fn from(value: ScheduleTaskTypeChoice) -> Self {
        match value {
            ScheduleTaskTypeChoice::Notification => ScheduledTaskType::Notification,
            ScheduleTaskTypeChoice::Dissolution => ScheduledTaskType::Dissolution,
            ScheduleTaskTypeChoice::DataCleanup => ScheduledTaskType::DataCleanup,
            ScheduleTaskTypeChoice::RecurringRecruitment => ScheduledTaskType::RecurringRecruitment,
            ScheduleTaskTypeChoice::Dismissal => ScheduledTaskType::Dismissal,
            ScheduleTaskTypeChoice::AutoRecruitmentRotation => {
                ScheduledTaskType::AutoRecruitmentRotation
            }
            ScheduleTaskTypeChoice::AutoMatching => ScheduledTaskType::AutoMatching,
        }
    }
}
