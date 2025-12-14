use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::services::recruitment::schedule::ScheduleDisplayService;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use tracing::{debug, warn};

/// 募集スケジュールの入力候補を取得するファサード
///
/// - RLS/トランザクション管理
/// - Repositoryでスケジュール一覧取得（作成者フィルタ）
/// - 表示整形（サービス）
pub async fn get_schedules_for_autocomplete(
    ctx: PoiseContext<'_>,
) -> Vec<AutocompleteChoice> {
    // ギルドIDを取得（サーバー外では空）
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            debug!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };

    let user_id = ctx.author().id.get() as i64;
    let app_state = &ctx.data().app_state;
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
    let schedule_repo = BattleRecruitmentScheduleRepository::new();
    let schedules = match schedule_repo.find_by_created_by(&txn, user_id).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "スケジュールの取得に失敗しました");
            let _ = txn.rollback().await;
            return vec![];
        }
    };

    let _ = txn.commit().await;

    // 表示用へ整形
    ScheduleDisplayService::to_autocomplete(&schedules)
}
