use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::battle_style_repository::{BattleStyleRepository, SeaOrmBattleStyleRepository};
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::repository::quests_repository::QuestRepository;
use crate::services::schedule::{convert_local_days_and_time_to_utc, RecruitmentScheduleService};
use crate::services::timezone_service::TimezoneService;
use crate::types::{PoiseContext, Result};
use chrono::NaiveTime;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

use super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

/// マルチ募集スケジュールを作成
///
/// 指定した曜日と時刻に自動的にマルチ募集を投稿するスケジュールを作成します。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-create",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集作成"),
    description_localized("ja", "指定した曜日と時刻に自動的にマルチ募集を投稿するスケジュールを作成します"),
)]
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
    #[description = "Target days (comma-separated. e.g., 月,水,金 / 火,木 / 毎日)"]
    #[description_localized("ja", "対象曜日（月火水木金土日から選択。カンマ区切り。例: 月,水,金 / 火,木 / 土,日 / 毎日）")]
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
    #[description = "Recruitment start day offset (0=same day, 1=day before, default: 1)"]
    #[description_localized("ja", "募集開始日オフセット（0=当日、1=前日、2=二日前、デフォルト: 1）")]
    #[min = 0]
    #[max = 7]
    recruit_start_day_offset: Option<i64>,
    #[name_localized("ja", "備考")]
    #[description = "Note (optional)"]
    #[description_localized("ja", "備考（省略可）")]
    note: Option<String>,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        }
    })?;

    let user_id = ctx.author().id;

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        "定期募集作成コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;

    // クエスト名からIDと詳細情報を取得
    let quest_repo = SeaOrmQuestRepository::new();
    let search_results = quest_repo
        .search_by_name_or_alias(app_state.guild_db(), &quest)
        .await?;

    let quest_search_result = search_results
        .first()
        .ok_or_else(|| crate::types::AppError::NotFound(format!(
            "クエスト '{}' が見つかりませんでした",
            quest
        )))?;

    let quest_id = quest_search_result.quest_id;

    // クエストの詳細情報を取得してデフォルトのbattle_style_idを取得
    let quest_detail = quest_repo
        .get_by_target_id(app_state.guild_db(), quest_id)
        .await?
        .ok_or_else(|| crate::types::AppError::NotFound(format!(
            "クエストID {} の詳細情報が見つかりませんでした",
            quest_id
        )))?;

    // battle_style_idの決定（指定されていればそれを使用、なければクエストのデフォルト値）
    let battle_style_id = battle_style.unwrap_or(quest_detail.default_battle_style_id);

    // バトルスタイル名を取得
    let battle_style_repo = SeaOrmBattleStyleRepository::new();
    let battle_style_detail = battle_style_repo
        .get_by_id(app_state.guild_db(), battle_style_id)
        .await?
        .ok_or_else(|| crate::types::AppError::NotFound(format!(
            "バトルスタイルID {} が見つかりませんでした",
            battle_style_id
        )))?;

    // ギルドのタイムゾーンを取得
    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    let timezone_service = TimezoneService::new(timezone_repo);
    let timezone = timezone_service
        .get_guild_timezone(app_state.guild_db(), guild_id.get() as i64)
        .await?;

    info!(
        guild_id = guild_id.get(),
        timezone = %timezone,
        "ギルドのタイムゾーンを取得しました"
    );

    // クエスト開始時刻をパース（ローカル時刻）
    let quest_start_time_local = parse_time(&quest_start_time)?;

    // 募集開始時刻をパース（ローカル時刻、必須）
    let recruit_start_time_local = parse_time(&recruit_start_time)?;

    let recruit_day_offset = recruit_start_day_offset.unwrap_or(1) as i32;

    // 曜日をパース（ローカル曜日）
    let local_day_of_weeks = parse_days(&days)?;

    // バリデーション（ローカル時刻で実施）
    let service = RecruitmentScheduleService::new();
    service.validate_schedule_input(
        &local_day_of_weeks,
        quest_start_time_local,
        recruit_day_offset,
        Some(recruit_start_time_local),
    )?;

    // ローカル時刻・曜日をUTCに変換
    let (utc_quest_days, quest_start_time_tt) =
        convert_local_days_and_time_to_utc(&local_day_of_weeks, quest_start_time_local, timezone)?;

    let (_utc_recruit_days, recruit_start_time_tt_val) =
        convert_local_days_and_time_to_utc(&local_day_of_weeks, recruit_start_time_local, timezone)?;
    let recruit_start_time_tt = Some(recruit_start_time_tt_val);

    info!(
        quest_local_time = %quest_start_time,
        quest_utc_time = format!("{:02}:{:02}", quest_start_time_tt.hour(), quest_start_time_tt.minute()),
        local_days = ?local_day_of_weeks,
        utc_days = ?utc_quest_days,
        "ローカル時刻・曜日をUTCに変換しました"
    );

    // UTC曜日を使用（クエスト開始と募集開始で異なる場合があるが、通常は同じ曜日）
    // ここではクエスト開始の曜日を使用
    let day_of_weeks = utc_quest_days;

    // スケジュールを作成
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    // マルチ募集チャンネルを取得（channel_type = 2）
    let guild_channel_repo = GuildChannelRepository::new();
    let guild_channel = guild_channel_repo
        .get_by_guild_and_type_with_txn(&txn, guild_id.get() as i64, 2)
        .await?
        .ok_or_else(|| {
            crate::types::AppError::Business {
                message: format!(
                    "マルチ募集チャンネルが登録されていません。\n\n\
                    定期募集を作成するには、先に管理者に `/チャンネル登録` コマンドで\
                    マルチ募集チャンネルを登録してもらってください。"
                ),
            }
        })?;

    // マルチ募集チャンネルのIDを使用
    let channel_id = guild_channel.channel_id;

    let schedule_repo = BattleRecruitmentScheduleRepository::new();

    match schedule_repo
        .create_with_txn(
            &txn,
            name.clone(),
            guild_id.get() as i64,
            channel_id,
            quest_id,
            battle_style_id,
            quest_start_time_tt,
            recruit_day_offset,
            recruit_start_time_tt,
            None, // max_participants はクエストごとの設定を使用
            note.clone(),
            user_id.get() as i64,
            day_of_weeks.clone(),
        )
        .await
    {
        Ok((schedule, days_models)) => {
            txn.commit().await?;

            info!(
                schedule_id = schedule.id,
                guild_id = guild_id.get(),
                "定期募集スケジュールを作成しました"
            );

            // 曜日を文字列に変換（ローカル曜日を表示）
            let days_str = format_days(&local_day_of_weeks);

            let embed = CreateEmbed::default()
                .title("✅ 定期募集スケジュールを作成しました")
                .description(format!(
                    "**スケジュール名**: {}\n\
                     **スケジュールID**: {}\n\
                     **クエスト**: {} (ID: {})\n\
                     **マルチ攻略方法**: {}\n\
                     **対象曜日**: {} ({}タイムゾーン)\n\
                     **クエスト開始時刻**: {}\n\
                     **募集開始**: {}日前の{}\n\
                     **備考**: {}\n\
                     **作成者**: <@{}>\n\n\
                     このスケジュールに基づいて、自動的に募集が投稿されます。\n\
                     参加人数はクエストごとの設定を使用します。",
                    name,
                    schedule.id,
                    quest,
                    quest_id,
                    battle_style_detail.display_name,
                    days_str,
                    timezone,
                    quest_start_time,
                    recruit_day_offset,
                    recruit_start_time,
                    note.as_ref().unwrap_or(&"-".to_string()),
                    user_id.get()
                ))
                .color(0x00ff00)
                .footer(CreateEmbedFooter::new(format!(
                    "登録された曜日: {} 件",
                    days_models.len()
                )));

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "定期募集スケジュールの作成に失敗しました");
            return Err(e);
        }
    }

    Ok(())
}

/// 時刻文字列をパース（HH:MM形式）
fn parse_time(time_str: &str) -> Result<NaiveTime> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(crate::types::AppError::Business {
            message: format!("無効な時刻形式です: {}（HH:MM形式で指定してください）", time_str),
        });
    }

    let hour = parts[0].parse::<u32>().map_err(|_| {
        crate::types::AppError::Business {
            message: format!("無効な時刻です: {}", time_str),
        }
    })?;

    let minute = parts[1].parse::<u32>().map_err(|_| {
        crate::types::AppError::Business {
            message: format!("無効な時刻です: {}", time_str),
        }
    })?;

    NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| {
        crate::types::AppError::Business {
            message: format!("無効な時刻です: {}", time_str),
        }
    })
}

/// 曜日文字列をパース
fn parse_days(days_str: &str) -> Result<Vec<i32>> {
    let mut result = Vec::new();

    for day in days_str.split(',') {
        let day = day.trim();
        let day_num = match day {
            "毎日" | "everyday" => 0,
            "月" | "月曜" | "月曜日" | "mon" | "monday" => 1,
            "火" | "火曜" | "火曜日" | "tue" | "tuesday" => 2,
            "水" | "水曜" | "水曜日" | "wed" | "wednesday" => 3,
            "木" | "木曜" | "木曜日" | "thu" | "thursday" => 4,
            "金" | "金曜" | "金曜日" | "fri" | "friday" => 5,
            "土" | "土曜" | "土曜日" | "sat" | "saturday" => 6,
            "日" | "日曜" | "日曜日" | "sun" | "sunday" => 7,
            _ => {
                return Err(crate::types::AppError::Business {
                    message: format!("無効な曜日です: {}", day),
                })
            }
        };
        result.push(day_num);
    }

    Ok(result)
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
