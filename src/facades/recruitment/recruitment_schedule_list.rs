use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
use crate::services::recruitment::schedule::ScheduleDisplayService;
use crate::services::schedule::schedule_query_service::ScheduleQueryService;
use crate::services::timezone_service::TimezoneService;
use crate::types::discord::AutocompleteOption;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use tracing::warn;

/// 募集スケジュールの入力候補を取得するファサード
///
/// - RLS/トランザクション管理
/// - Repositoryでスケジュール一覧取得（作成者フィルタ）
/// - 表示整形（サービス）
///
/// # 引数
/// * `conn` - データベース接続
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
pub async fn get_schedules_for_autocomplete(
    conn: &DatabaseConnection,
    guild_id: i64,
    user_id: i64,
) -> Vec<AutocompleteOption> {

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
    let schedule_query_service = ScheduleQueryService::new();
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
    let timezone_repo = SeaOrmGuildSettingsRepository::new();
    let timezone_service = TimezoneService::new(Arc::new(timezone_repo));
    let timezone = match timezone_service.get_guild_timezone(conn, guild_id).await {
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
