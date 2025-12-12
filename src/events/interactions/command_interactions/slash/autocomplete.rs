use crate::facades::recruitment::battle_style_list;
use crate::facades::recruitment::quest_list;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::services::timezone_service::TimezoneService;
use crate::types::PoiseContext;
use futures::Stream;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use tracing::{debug, warn};

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete<'a>(
    ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    quest_list::search_quests_for_autocomplete(ctx, partial).await
}

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    battle_style_list::get_battle_styles_for_autocomplete(ctx).await
}

/// タイムゾーンの入力候補を取得
pub async fn timezone_auto_complete(
    _ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    TimezoneService::get_timezones_for_autocomplete(partial)
}

/// 募集スケジュールの入力候補を取得
pub async fn recruitment_schedule_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    // ギルドIDを取得
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

    // トランザクションを開始してRLSを設定
    let txn = match conn.begin().await {
        Ok(t) => t,
        Err(e) => {
            warn!(error = %e, "トランザクションの開始に失敗しました");
            return vec![];
        }
    };

    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        warn!(error = %e, "RLSの設定に失敗しました");
        let _ = txn.rollback().await;
        return vec![];
    }

    // スケジュール一覧を取得（自分が作成したもの）
    let schedule_repo = BattleRecruitmentScheduleRepository::new();
    let schedules = match schedule_repo
        .find_by_created_by(&txn, user_id)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "スケジュールの取得に失敗しました");
            let _ = txn.rollback().await;
            return vec![];
        }
    };

    let _ = txn.commit().await;

    // AutocompleteChoiceのリストを作成
    let mut choices = Vec::new();
    for (schedule, days) in schedules.iter().take(25) {
        // 最大25件
        // 曜日を文字列化
        let days_str = format_days_for_display(days);

        // 時刻をフォーマット
        let time_str = format!(
            "{:02}:{:02}",
            schedule.quest_start_time.hour(),
            schedule.quest_start_time.minute()
        );

        // 有効/無効の表示
        let status = if schedule.is_enabled { "⭕" } else { "❌" };

        // 表示名を作成（例: "⭕ 天元21時 (ID:1 火木土 22:00)"）
        let display_name = format!(
            "{} {} (ID:{} {} {})",
            status, schedule.name, schedule.id, days_str, time_str
        );

        choices.push(AutocompleteChoice::new(
            display_name,
            schedule.id as i64,
        ));
    }

    choices
}

/// 曜日リストを表示用にフォーマット
fn format_days_for_display(days: &[crate::models::entities::battle_recruitment_schedule_days::Model]) -> String {
    if days.is_empty() {
        return "なし".to_string();
    }

    // 毎日かチェック
    if days.iter().any(|d| d.day_of_week == 0) {
        return "毎日".to_string();
    }

    // 曜日をソート
    let mut day_numbers: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
    day_numbers.sort();

    // 曜日を文字列に変換
    let day_strs: Vec<String> = day_numbers
        .iter()
        .map(|&day| match day {
            1 => "月",
            2 => "火",
            3 => "水",
            4 => "木",
            5 => "金",
            6 => "土",
            7 => "日",
            _ => "?",
        })
        .map(|s| s.to_string())
        .collect();

    day_strs.join("")
}
