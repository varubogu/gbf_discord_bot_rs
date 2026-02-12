use crate::repository::db_helper::set_current_guild_id;
use crate::services::recruitment::schedule::ScheduleDisplayService;
use crate::services::schedule::schedule_query_service::ScheduleQueryService;
use crate::services::timezone_service::TimezoneService;
use crate::types::AppState;
use crate::types::discord::AutocompleteOption;
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
    ScheduleDisplayService::to_autocomplete(&schedules, &timezone)
}
