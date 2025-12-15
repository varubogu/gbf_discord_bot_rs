use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::repository::quests_repository::QuestRepository;
use crate::services::schedule::convert_utc_days_and_time_to_local;
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use chrono::Timelike;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::info;

/// マルチ募集スケジュール一覧を表示
///
/// 登録されているマルチ募集スケジュールを表示します。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-list",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集一覧"),
    description_localized("ja", "登録されているマルチ募集スケジュールを表示します"),
)]
pub async fn recruitment_schedule_list(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "全員表示")]
    #[description = "Show all schedules (default: false, only yours)"]
    #[description_localized("ja", "全員のスケジュールを表示（デフォルト: false、自分のみ）")]
    show_all: Option<bool>,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        }
    })?;

    let user_id = ctx.author().id;
    let show_all = show_all.unwrap_or(false);

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        show_all = show_all,
        "定期募集一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let schedule_repo = BattleRecruitmentScheduleRepository::new();

    // スケジュールを取得
    let schedules = if show_all {
        schedule_repo
            .find_by_guild_id(&txn, guild_id.get() as i64)
            .await?
    } else {
        schedule_repo
            .find_by_created_by(&txn, user_id.get() as i64)
            .await?
    };

    txn.commit().await?;

    // ギルドのタイムゾーンを取得（UTC→ローカル変換用）
    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    let timezone_service = TimezoneService::new(timezone_repo);
    let timezone = timezone_service
        .get_guild_timezone(app_state.guild_db(), guild_id.get() as i64)
        .await?;

    if schedules.is_empty() {
        let embed = CreateEmbed::default()
            .title("📅 定期募集スケジュール一覧")
            .description(if show_all {
                "登録されているスケジュールはありません。\n\n`/recruitment-schedule-create` コマンドでスケジュールを作成してください。"
            } else {
                "あなたが作成したスケジュールはありません。\n\n`/recruitment-schedule-create` コマンドでスケジュールを作成してください。"
            })
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    // 最大10件に制限
    let display_count = schedules.len().min(10);
    let total_count = schedules.len();

    // クエスト名を取得するためのリポジトリ
    let quest_repo = SeaOrmQuestRepository::new();

    let mut description = String::new();
    for (i, (schedule, days)) in schedules.iter().take(display_count).enumerate() {
        // クエスト名を取得
        let quest_name = match quest_repo.get_by_target_id(app_state.guild_db(), schedule.quest_id).await {
            Ok(Some(quest)) => quest.name,
            _ => format!("クエストID {}", schedule.quest_id),
        };

        // UTC曜日・時刻をローカル曜日・時刻に変換
        let utc_days: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
        let (local_days, local_quest_time) = convert_utc_days_and_time_to_local(
            &utc_days,
            schedule.quest_start_time,
            timezone,
        )?;

        // 曜日を文字列に変換（ローカル曜日）
        let days_str = format_days(&local_days);

        // 募集開始時刻を表示（ローカル時刻）
        let recruit_time_str = if let Some(recruit_time_utc) = schedule.recruit_start_time {
            let (_, local_recruit_time) = convert_utc_days_and_time_to_local(
                &utc_days,
                recruit_time_utc,
                timezone,
            )?;
            format!("{:02}:{:02}", local_recruit_time.hour(), local_recruit_time.minute())
        } else {
            format!(
                "{:02}:{:02}（クエスト開始時刻と同じ）",
                local_quest_time.hour(),
                local_quest_time.minute()
            )
        };

        let status = if schedule.is_enabled {
            "✅ 有効"
        } else {
            "❌ 無効"
        };

        description.push_str(&format!(
            "{}. **{}** (ID: {}) {}\n\
                クエスト: {}\n\
                曜日: {} ({})\n\
                開始: {:02}:{:02}\n\
                募集: {}日前の{}\n\
                作成者: <@{}>\n\n",
            i + 1,
            schedule.name,
            schedule.id,
            status,
            quest_name,
            days_str,
            timezone,
            local_quest_time.hour(),
            local_quest_time.minute(),
            schedule.recruit_start_day_offset,
            recruit_time_str,
            schedule.created_by
        ));
    }

    if total_count > display_count {
        description.push_str(&format!(
            "\n*...他 {} 件のスケジュールがあります*",
            total_count - display_count
        ));
    }

    let title = if show_all {
        "📅 定期募集スケジュール一覧（全員）"
    } else {
        "📅 定期募集スケジュール一覧（自分のみ）"
    };

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(format!(
            "全 {} 件のスケジュール（{}件表示）",
            total_count, display_count
        )));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// 曜日を文字列に変換
fn format_days(days: &[i32]) -> String {
    let day_names: Vec<String> = days
        .iter()
        .map(|&d| match d {
            0 => "毎日".to_string(),
            1 => "月".to_string(),
            2 => "火".to_string(),
            3 => "水".to_string(),
            4 => "木".to_string(),
            5 => "金".to_string(),
            6 => "土".to_string(),
            7 => "日".to_string(),
            _ => format!("不明({})", d),
        })
        .collect();

    day_names.join(", ")
}
