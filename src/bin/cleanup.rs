use std::{env, path::Path, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Utc};
use gbf_discord_bot_rs::di::Repositories;
use gbf_discord_bot_rs::infrastructure::database::connection::sea_orm_connection::DatabaseConnectionManager;
use gbf_discord_bot_rs::services::maintenance::DataCleanupService;
use sea_orm::TransactionTrait;
use tracing::{error, info, warn};

/// クリーンアップ実行モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanupExecutionMode {
    /// 1回だけ実行して終了
    Once,
    /// 毎日指定時刻に定期実行
    Scheduler,
}

/// 定期実行スケジュール設定
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CleanupScheduleConfig {
    /// 実行時刻（UTC時）
    hour_utc: u32,
    /// 実行時刻（UTC分）
    minute_utc: u32,
    /// 起動直後にも1回実行するか
    run_on_startup: bool,
}

/// データクリーンアップバッチのエントリーポイント
///
/// 環境変数からDB接続情報を取得し、DataCleanupServiceを実行する。
/// `CLEANUP_EXECUTION_MODE=scheduler` の場合は毎日指定時刻に定期実行される。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_maintenance_env();

    // ログ初期化
    tracing_subscriber::fmt::init();

    let execution_mode = parse_execution_mode().map_err(|e| {
        error!(error = %e, "CLEANUP_EXECUTION_MODEの読み込みに失敗しました");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    match execution_mode {
        CleanupExecutionMode::Once => execute_cleanup_once().await,
        CleanupExecutionMode::Scheduler => {
            let schedule = parse_schedule_config().map_err(|e| {
                error!(error = %e, "定期実行設定の読み込みに失敗しました");
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
            })?;
            run_scheduler(schedule).await
        }
    }
}

/// 設定フォルダ配下の `.env.maintenance` を読み込む
fn load_maintenance_env() {
    // .env.maintenanceファイルを読み込み（開発環境用）
    // 本番環境では環境変数が直接設定されるため、エラーは無視
    let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
    let dotenv_path = Path::new(&config_folder).join(".env.maintenance");
    dotenv::from_path(dotenv_path).ok();
}

/// データクリーンアップを1回実行
async fn execute_cleanup_once() -> Result<(), Box<dyn std::error::Error>> {
    info!("データクリーンアップ処理を開始します");

    let manager = DatabaseConnectionManager::new().await.map_err(|e| {
        error!(error = %e, "データベース接続に失敗しました");
        e
    })?;
    let db = manager.connection().clone();

    // リポジトリ初期化
    let repositories = Repositories::new();

    // クリーンアップサービス初期化
    let cleanup_service = DataCleanupService::new(
        repositories.battle_recruitments,
        repositories.notification,
        repositories.scheduled_task,
    );

    // トランザクション開始
    let txn = db.begin().await.map_err(|e| {
        error!(error = %e, "トランザクション開始に失敗しました");
        e
    })?;

    // クリーンアップ実行
    match cleanup_service.execute(&txn).await {
        Ok(stats) => {
            // トランザクションコミット
            txn.commit().await.map_err(|e| {
                error!(error = %e, "トランザクションコミットに失敗しました");
                e
            })?;

            info!(
                recruitments = stats.deleted_recruitments,
                notifications = stats.deleted_notifications,
                tasks = stats.deleted_tasks,
                cleanup_before = %stats.cleanup_before,
                "データクリーンアップが正常に完了しました"
            );
            Ok(())
        }
        Err(e) => {
            // エラー時はトランザクションロールバック（自動）
            error!(error = %e, "データクリーンアップに失敗しました");
            Err(e.into())
        }
    }
}

/// 実行モードを環境変数から読み込む
fn parse_execution_mode() -> Result<CleanupExecutionMode, String> {
    match env::var("CLEANUP_EXECUTION_MODE")
        .unwrap_or_else(|_| "once".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "once" => Ok(CleanupExecutionMode::Once),
        "scheduler" => Ok(CleanupExecutionMode::Scheduler),
        value => Err(format!(
            "CLEANUP_EXECUTION_MODEが不正です: {value}（once または scheduler を指定してください）"
        )),
    }
}

/// 定期実行設定を環境変数から読み込む
fn parse_schedule_config() -> Result<CleanupScheduleConfig, String> {
    let hour_utc = parse_u32_env_with_max("CLEANUP_SCHEDULE_HOUR_UTC", 3, 23)?;
    let minute_utc = parse_u32_env_with_max("CLEANUP_SCHEDULE_MINUTE_UTC", 0, 59)?;
    let run_on_startup = parse_bool_env("CLEANUP_RUN_ON_STARTUP", false)?;

    Ok(CleanupScheduleConfig {
        hour_utc,
        minute_utc,
        run_on_startup,
    })
}

/// `u32` 型の環境変数を上限付きで読み込む
fn parse_u32_env_with_max(name: &str, default_value: u32, max_value: u32) -> Result<u32, String> {
    let raw = env::var(name).unwrap_or_else(|_| default_value.to_string());
    let parsed = raw
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{name}は0以上の整数で指定してください: {raw}"))?;

    if parsed > max_value {
        return Err(format!(
            "{name}は{max_value}以下で指定してください: {parsed}"
        ));
    }

    Ok(parsed)
}

/// `bool` 型の環境変数を読み込む
fn parse_bool_env(name: &str, default_value: bool) -> Result<bool, String> {
    let raw = match env::var(name) {
        Ok(value) => value,
        Err(_) => return Ok(default_value),
    };

    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!(
            "{name}は true/false, 1/0, yes/no, on/off のいずれかで指定してください: {raw}"
        )),
    }
}

/// 次回実行時刻を計算する
fn calculate_next_run(
    now: DateTime<Utc>,
    hour_utc: u32,
    minute_utc: u32,
) -> Result<DateTime<Utc>, String> {
    let today_target = now
        .date_naive()
        .and_hms_opt(hour_utc, minute_utc, 0)
        .ok_or_else(|| {
            format!("実行時刻の生成に失敗しました（hour={hour_utc}, minute={minute_utc}）")
        })?
        .and_utc();

    if today_target <= now {
        Ok(today_target + Duration::days(1))
    } else {
        Ok(today_target)
    }
}

/// 定期実行モードを開始
async fn run_scheduler(schedule: CleanupScheduleConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        hour_utc = schedule.hour_utc,
        minute_utc = schedule.minute_utc,
        run_on_startup = schedule.run_on_startup,
        "データクリーンアップ定期実行モードを開始します"
    );

    if schedule.run_on_startup
        && let Err(e) = execute_cleanup_once().await
    {
        error!(error = %e, "起動直後のデータクリーンアップに失敗しました");
    }

    loop {
        let now = Utc::now();
        let next_run =
            calculate_next_run(now, schedule.hour_utc, schedule.minute_utc).map_err(|e| {
                error!(error = %e, "次回実行時刻の計算に失敗しました");
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
            })?;

        let wait_duration = (next_run - now)
            .to_std()
            .unwrap_or_else(|_| StdDuration::from_secs(0));

        info!(
            next_run = %next_run,
            wait_seconds = wait_duration.as_secs(),
            "次回実行まで待機します"
        );

        tokio::select! {
            _ = tokio::time::sleep(wait_duration) => {
                if let Err(e) = execute_cleanup_once().await {
                    error!(error = %e, "データクリーンアップの定期実行に失敗しました");
                }
            }
            _ = wait_for_shutdown_signal() => {
                info!("停止シグナルを受信したため、定期実行モードを終了します");
                return Ok(());
            }
        }
    }
}

/// 停止シグナルを待機する
#[cfg(unix)]
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    match signal(SignalKind::terminate()) {
        Ok(mut sigterm) => {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = sigterm.recv() => {}
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                "SIGTERMハンドラ初期化に失敗したため、Ctrl+C待機のみを利用します"
            );
            if let Err(ctrl_c_err) = tokio::signal::ctrl_c().await {
                error!(error = %ctrl_c_err, "Ctrl+C待機に失敗しました");
            }
        }
    }
}

/// 停止シグナルを待機する（非Unix）
#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        error!(error = %e, "Ctrl+C待機に失敗しました");
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_next_run;
    use chrono::{TimeZone, Utc};

    #[test]
    fn 次回実行時刻_当日未来時刻を返す() {
        let now = Utc
            .with_ymd_and_hms(2026, 4, 18, 2, 30, 0)
            .single()
            .expect("現在時刻の生成に失敗しました");

        let next_run = calculate_next_run(now, 3, 0).expect("次回実行時刻の計算に失敗しました");
        let expected = Utc
            .with_ymd_and_hms(2026, 4, 18, 3, 0, 0)
            .single()
            .expect("期待時刻の生成に失敗しました");

        assert_eq!(next_run, expected);
    }

    #[test]
    fn 次回実行時刻_同時刻なら翌日を返す() {
        let now = Utc
            .with_ymd_and_hms(2026, 4, 18, 3, 0, 0)
            .single()
            .expect("現在時刻の生成に失敗しました");

        let next_run = calculate_next_run(now, 3, 0).expect("次回実行時刻の計算に失敗しました");
        let expected = Utc
            .with_ymd_and_hms(2026, 4, 19, 3, 0, 0)
            .single()
            .expect("期待時刻の生成に失敗しました");

        assert_eq!(next_run, expected);
    }
}
