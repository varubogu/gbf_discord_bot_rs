use crate::events::converters::to_create_embed;
use crate::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use crate::services::recruitment::schedule::ScheduleDisplayService;
use crate::types::{PoiseContext, Result};
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
            recruit_start_day_offset.map(|offset| offset as i32),
            note,
            dismissal_times,
        )
        .await;

    let schedule_data = result.map_err(|e| {
        error!(error = %e, "定期募集スケジュールの作成に失敗しました");
        e
    })?;

    let embed_content = ScheduleDisplayService::build_creation_embed(&schedule_data, user_id.get());
    let embed = to_create_embed(&embed_content);
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
