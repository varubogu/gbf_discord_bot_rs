use thiserror::Error;

/// スケジュール実行系の運用エラー
#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error("スケジューラーの初期化に失敗しました: {0}")]
    SchedulerInitialization(String),

    #[error("ジョブの作成に失敗しました: {0}")]
    JobCreation(String),

    #[error("ジョブの登録に失敗しました: {0}")]
    JobRegistration(String),

    #[error("スケジューラーの開始に失敗しました: {0}")]
    SchedulerStart(String),

    #[error("スケジューラーの停止に失敗しました: {0}")]
    SchedulerShutdown(String),

    #[error("起動時修復に失敗しました")]
    StartupRepairFailed,

    #[error("スケジュールディスパッチに失敗しました")]
    DispatchFailed,
}
