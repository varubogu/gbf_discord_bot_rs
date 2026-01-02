use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use crate::services::recruitment::schedule::{OffsetCalculatorService, ScheduleDisplayService};
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::CreateEmbed;
use std::sync::Arc;
use tracing::{error, info};

use super::super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

/// マルチ募集スケジュールを作成
///
/// 指定した曜日と時刻に自動的にマルチ募集を投稿するスケジュールを作成します。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-create",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集作成"),
    description_localized(
        "ja",
        "指定した曜日と時刻に自動的にマルチ募集を投稿するスケジュールを作成します"
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn recruitment_schedule_create(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "スケジュール名")]
    #[description = "Schedule name (e.g., 天元21時)"]
    #[description_localized("ja", "スケジュール名（例: 天元21時）")]
    name: String,
    #[autocomplete = "quest_auto_complete"]
    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    quest: String,
    #[name_localized("ja", "クエスト開始時刻")]
    #[description = "Quest start time (e.g., 22:00)"]
    #[description_localized("ja", "クエスト開始時刻（例: 22:00）")]
    quest_start_time: String,
    #[name_localized("ja", "対象曜日")]
    #[description = "Target days (comma/space separated or continuous. e.g., 月,水,金 / 火 木 / 月火水 / 毎日)"]
    #[description_localized(
        "ja",
        "対象曜日（月火水木金土日から選択。区切り文字または連続入力。例: 月,水,金 / 火 木 / 月火水 / 金土日 / 毎日）"
    )]
    days: String,
    #[name_localized("ja", "募集開始時刻")]
    #[description = "Recruitment start time (e.g., 20:00)"]
    #[description_localized("ja", "募集開始時刻（例: 20:00）")]
    recruit_start_time: String,
    #[autocomplete = "battle_style_auto_complete"]
    #[name_localized("ja", "マルチ攻略方法")]
    #[description = "battle style (optional, uses quest default if not specified)"]
    #[description_localized("ja", "マルチ攻略方法（省略時はクエストのデフォルト値を使用）")]
    battle_style: Option<i32>,
    #[name_localized("ja", "募集開始日オフセット")]
    #[description = "Recruitment start day offset (0=same day, 1=day before, default: auto)"]
    #[description_localized(
        "ja",
        "募集開始日オフセット（0=当日、1=前日、2=二日前、省略時は自動判定）"
    )]
    #[min = 0]
    #[max = 7]
    recruit_start_day_offset: Option<i64>,
    #[name_localized("ja", "備考")]
    #[description = "Note (optional)"]
    #[description_localized("ja", "備考（省略可）")]
    note: Option<String>,
    #[name_localized("ja", "解散時刻")]
    #[description = "dismissal times (comma-separated, max 3)"]
    #[description_localized("ja", "解散時刻（カンマ区切り、最大3つ。例: 1時間前, 21:00, 2日前）")]
    dismissal_times: Option<String>,
) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

    let user_id = ctx.author().id;

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        "定期募集作成コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = ctx.data().app_state.clone();

    // タイムゾーンを取得
    let timezone_facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let timezone = timezone_facade.get_timezone(guild_id.get() as i64).await?;

    // クエスト開始時刻をパース（HH:MM厳格モード）
    let quest_options = DateTimeParseOptions::strict_hhmm_only(timezone);
    let quest_results = parse_datetime(&quest_start_time, &quest_options)?;
    let quest_time = match &quest_results[0] {
        ParsedDateTime::Time(t) => *t,
        _ => {
            return Err(crate::types::AppError::Business {
                message: "クエスト開始時刻はHH:MM形式で指定してください".to_string(),
            });
        }
    };

    // 募集開始時刻をパース（相対時刻もサポート）
    let recruit_options = DateTimeParseOptions::for_schedule_start_time(timezone, quest_time);
    let recruit_results = parse_datetime(&recruit_start_time, &recruit_options)?;

    // ParsedDateTimeからNaiveTimeを取得
    let recruit_time = match &recruit_results[0] {
        ParsedDateTime::Time(t) => *t,
        ParsedDateTime::Relative {
            days,
            hours,
            minutes,
        } => {
            // 相対時刻の場合、クエスト開始時刻から計算
            use chrono::{Duration, NaiveDate};

            // 相対時刻を合計分数に変換（マイナスにする）
            let total_minutes = -(days * 24 * 60 + hours * 60 + minutes);
            let duration = Duration::minutes(total_minutes as i64);

            // 仮の日付を使ってDateTime演算を行い、時刻部分を取得
            let dummy_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
            let base_datetime = dummy_date.and_time(quest_time);
            let result_datetime = base_datetime + duration;

            result_datetime.time()
        }
        ParsedDateTime::Absolute(dt) => {
            // 絶対日時の場合、タイムゾーンに変換して時刻部分を取得
            dt.with_timezone(&timezone).time()
        }
        #[allow(unreachable_patterns)]
        _ => {
            return Err(crate::types::AppError::Business {
                message: "募集開始時刻は時刻または相対時刻で指定してください".to_string(),
            });
        }
    };

    // オフセットのデフォルト値を決定
    let default_offset = if let Some(offset) = recruit_start_day_offset {
        offset as i32
    } else {
        // オフセット指定なしの場合、時刻から自動判定
        OffsetCalculatorService::determine_default_offset(recruit_time, quest_time)
    };

    // Facade層を呼び出し
    let facade = RecruitmentScheduleFacade::new(Arc::new(app_state));
    let result = facade
        .create_recruitment_schedule(
            guild_id.get(),
            user_id.get(),
            name,
            &quest,
            &quest_start_time,
            &days,
            &recruit_start_time,
            battle_style,
            default_offset,
            note,
            dismissal_times,
        )
        .await;

    let schedule_data = result.map_err(|e| {
        error!(error = %e, "定期募集スケジュールの作成に失敗しました");
        e
    })?;

    let embed: CreateEmbed =
        ScheduleDisplayService::build_creation_embed(&schedule_data, user_id.get());
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
