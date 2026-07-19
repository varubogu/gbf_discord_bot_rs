use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::entities::guild_master::battle_recruitment_schedule_days;
use crate::models::entities::guild_master::battle_recruitment_schedules;
use crate::presenter::{ScheduleDisplayInfo, SchedulePresenter};
use crate::services::schedule::schedule_query_service::ScheduleQueryService;
use crate::services::timezone_service::TimezoneService;
use crate::types::AppState;
use crate::types::discord::AutocompleteOption;
use chrono_tz::Tz;
use sea_orm::TransactionTrait;
use tracing::warn;

/// 募集スケジュールの入力候補を取得するファサード
///
/// - RLS/トランザクション管理
/// - Repositoryでスケジュール一覧取得（作成者フィルタ）
/// - 表示整形（サービス）
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
pub async fn get_schedules_for_autocomplete(
    app_state: &AppState,
    guild_id: i64,
    user_id: i64,
) -> Vec<AutocompleteOption> {
    let conn = app_state.guild_db();

    // Tx開始
    let txn = match conn.begin().await {
        Ok(t) => t,
        Err(e) => {
            warn!(error = %e, "トランザクションの開始に失敗しました");
            return vec![];
        }
    };

    // RLS用セッション変数を設定
    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        warn!(error = %e, "RLSの設定に失敗しました");
        let _ = txn.rollback().await;
        return vec![];
    }

    // スケジュール一覧（自分が作成したもの）
    let schedule_query_service = ScheduleQueryService::new(
        app_state.repositories.battle_recruitment_schedule,
        app_state.repositories.quest,
        app_state.repositories.battle_recruitment_schedule_dismissal,
        app_state.repositories.notification,
    );
    let schedules = match schedule_query_service
        .get_schedules_by_user(&txn, user_id)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "スケジュールの取得に失敗しました");
            let _ = txn.rollback().await;
            return vec![];
        }
    };

    // タイムゾーンを取得（トランザクション内）
    let guild_settings_repo = app_state.repositories.guild_settings;
    let timezone_service = TimezoneService::new(guild_settings_repo);
    let timezone = match timezone_service
        .get_guild_timezone_with_txn(&txn, guild_id)
        .await
    {
        Ok(tz) => tz,
        Err(e) => {
            warn!(error = %e, "タイムゾーンの取得に失敗しました");
            let _ = txn.rollback().await;
            return vec![];
        }
    };

    let _ = txn.commit().await;

    // 表示用へ整形（タイムゾーンを渡す）
    to_autocomplete(&schedules, &timezone)
}

/// 曜日リストを表示用にフォーマット
fn format_days_for_display(days: &[battle_recruitment_schedule_days::Model]) -> String {
    if days.is_empty() {
        return "なし".to_string();
    }

    // 毎日かチェック（0: 毎日）
    if days.iter().any(|d| d.day_of_week == 0) {
        return "毎日".to_string();
    }

    // 曜日番号を抽出してSchedulePresenterに委譲
    let day_numbers: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
    SchedulePresenter::format_days(&day_numbers)
}

/// エンティティからScheduleDisplayInfoに変換する
fn to_display_info(
    schedule: &battle_recruitment_schedules::Model,
    days: &[battle_recruitment_schedule_days::Model],
    timezone: &Tz,
) -> ScheduleDisplayInfo {
    let days_display = format_days_for_display(days);
    let time_display = SchedulePresenter::format_time_with_timezone(
        schedule.quest_start_time.hour() as u32,
        schedule.quest_start_time.minute() as u32,
        timezone,
    );

    ScheduleDisplayInfo {
        id: schedule.id,
        name: schedule.name.clone(),
        quest_name: String::new(), // 後で設定される場合がある
        days_display,
        time_display,
        is_enabled: schedule.is_enabled,
    }
}

/// オートコンプリート用の候補へ変換（最大25件）
/// タイムゾーンを適用して時刻を表示
fn to_autocomplete(
    schedules: &[(
        battle_recruitment_schedules::Model,
        Vec<battle_recruitment_schedule_days::Model>,
    )],
    timezone: &Tz,
) -> Vec<AutocompleteOption> {
    let display_infos: Vec<ScheduleDisplayInfo> = schedules
        .iter()
        .take(25)
        .map(|(schedule, days)| to_display_info(schedule, days, timezone))
        .collect();

    SchedulePresenter::create_schedule_autocomplete(&display_infos)
}
