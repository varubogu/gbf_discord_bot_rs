use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::{
    BattleRecruitmentScheduleDismissalRepository, BattleRecruitmentScheduleRepository,
    ScheduledTaskRecurringRecruitmentRepository, ScheduledTaskRepository,
};
use crate::repository::quests_repository::QuestRepository;
use crate::services::recruitment::schedule::DaysParserService;
use crate::services::schedule::{RecruitmentScheduleService, convert_local_days_and_time_to_utc};
use crate::services::unified_datetime_parser::{
    parse_datetime, DateTimeParseOptions, ParsedDateTime,
};
use crate::types::{AppError, Result};
use chrono::{Duration, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use sea_orm::DatabaseTransaction;
use sea_orm::entity::prelude::TimeTime;
use tracing::{debug, info};

/// スケジュール作成結果
#[derive(Debug, Clone)]
pub struct ScheduleCreationResult {
    pub schedule_id: i64,
    pub schedule_name: String,
    pub quest_name: String,
    pub quest_id: i32,
    pub battle_style_name: String,
    pub battle_style_id: i32,
    pub days_display: String,
    pub quest_start_time: String,
    pub recruit_start_day_offset: i32,
    pub recruit_start_time: String,
    pub note: Option<String>,
    pub timezone: Tz,
    pub dismissal_times: Option<String>,
}

/// スケジュール作成サービス
///
/// 定期募集スケジュールの作成ビジネスロジックを担当するサービス。
pub struct ScheduleCreateService {
    days_parser: DaysParserService,
    schedule_service: RecruitmentScheduleService,
}

impl ScheduleCreateService {
    pub fn new() -> Self {
        Self {
            days_parser: DaysParserService::new(),
            schedule_service: RecruitmentScheduleService::new(),
        }
    }

    /// 定期募集スケジュールを作成
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `guild_id`: ギルドID
    /// - `user_id`: ユーザーID
    /// - `name`: スケジュール名
    /// - `quest_alias`: クエスト名またはエイリアス
    /// - `quest_start_time`: クエスト開始時刻（ローカル時刻、HH:MM形式）
    /// - `days`: 対象曜日文字列
    /// - `recruit_start_time`: 募集開始時刻（ローカル時刻、HH:MM形式）
    /// - `battle_style_id`: バトルスタイルID（省略可）
    /// - `recruit_day_offset`: 募集開始日オフセット
    /// - `note`: 備考（省略可）
    /// - `timezone`: タイムゾーン
    ///
    /// # 戻り値
    /// スケジュール作成結果
    #[allow(clippy::too_many_arguments)]
    pub async fn create_schedule(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        name: String,
        quest_alias: &str,
        quest_start_time: &str,
        days: &str,
        recruit_start_time: &str,
        battle_style_id: Option<i32>,
        recruit_day_offset: i32,
        note: Option<String>,
        dismissal_times: Option<String>,
        timezone: Tz,
    ) -> Result<ScheduleCreationResult> {
        // 1. クエスト検索・取得
        let (quest_id, quest_name, default_battle_style_id) =
            self.find_and_get_quest(txn, quest_alias).await?;

        // 2. バトルスタイル決定・取得
        let final_battle_style_id = battle_style_id.unwrap_or(default_battle_style_id);
        let battle_style_name = self
            .get_battle_style_name(txn, final_battle_style_id)
            .await?;

        // 3. 時刻・曜日パース
        // クエスト開始時刻（HH:MM厳格モード）
        let quest_options = DateTimeParseOptions::strict_hhmm_only(timezone);
        let quest_results = parse_datetime(quest_start_time, &quest_options)?;
        let quest_start_time_local = match &quest_results[0] {
            ParsedDateTime::Time(t) => *t,
            _ => {
                return Err(AppError::Business {
                    message: "クエスト開始時刻はHH:MM形式で指定してください".to_string(),
                })
            }
        };

        // 募集開始時刻（相対時刻もサポート）
        let recruit_options =
            DateTimeParseOptions::for_schedule_start_time(timezone, quest_start_time_local);
        let recruit_results = parse_datetime(recruit_start_time, &recruit_options)?;
        let recruit_start_time_local = match &recruit_results[0] {
            ParsedDateTime::Time(t) => *t,
            ParsedDateTime::Relative { days, hours, minutes } => {
                // 相対時刻の場合、クエスト開始時刻から計算
                use chrono::{Duration, NaiveDate};

                let total_minutes = -(days * 24 * 60 + hours * 60 + minutes);
                let duration = Duration::minutes(total_minutes as i64);

                // 仮の日付を使ってDateTime演算を行い、時刻部分を取得
                let dummy_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                let base_datetime = dummy_date.and_time(quest_start_time_local);
                let result_datetime = base_datetime + duration;

                result_datetime.time()
            }
            _ => {
                return Err(AppError::Business {
                    message: "募集開始時刻は時刻または相対時刻で指定してください".to_string(),
                })
            }
        };

        let local_day_of_weeks = self.days_parser.parse_days_input(days)?;

        // 4. バリデーション
        self.schedule_service.validate_schedule_input(
            &local_day_of_weeks,
            quest_start_time_local,
            recruit_day_offset,
            Some(recruit_start_time_local),
        )?;

        // 6. UTC変換
        let (utc_quest_days, quest_start_time_utc) = convert_local_days_and_time_to_utc(
            &local_day_of_weeks,
            quest_start_time_local,
            timezone,
        )?;
        let (_, recruit_start_time_utc) = convert_local_days_and_time_to_utc(
            &local_day_of_weeks,
            recruit_start_time_local,
            timezone,
        )?;

        info!(
            quest_local_time = %quest_start_time,
            quest_utc_time = format!("{:02}:{:02}", quest_start_time_utc.hour(), quest_start_time_utc.minute()),
            local_days = ?local_day_of_weeks,
            utc_days = ?utc_quest_days,
            "ローカル時刻・曜日をUTCに変換しました"
        );

        // 7. チャンネル取得（マルチ募集チャンネル: channel_type = 2）
        let channel_id = self.get_recruitment_channel(txn, guild_id).await?;

        // 8. スケジュール保存
        let schedule_repo = BattleRecruitmentScheduleRepository::new();
        let (schedule, days) = schedule_repo
            .create_with_txn(
                txn,
                name.clone(),
                guild_id,
                channel_id,
                quest_id,
                final_battle_style_id,
                quest_start_time_utc,
                recruit_day_offset,
                Some(recruit_start_time_utc),
                None, // max_participants はクエストごとの設定を使用
                note.clone(),
                user_id,
                utc_quest_days.clone(),
            )
            .await?;

        info!(
            schedule_id = schedule.id,
            guild_id = guild_id,
            "定期募集スケジュールを作成しました"
        );

        // 9. 次回実行日時を計算してscheduled_tasksに登録
        self.create_next_scheduled_task(txn, &schedule, &days)
            .await?;

        // 10. 解散時刻を登録（指定されている場合）
        if let Some(ref dismissal_times_str) = dismissal_times {
            self.save_dismissal_times(
                txn,
                schedule.id,
                dismissal_times_str,
                quest_start_time_local,
                timezone,
            )
            .await?;
        }

        // 11. 結果データ作成
        Ok(ScheduleCreationResult {
            schedule_id: schedule.id as i64,
            schedule_name: name,
            quest_name,
            quest_id,
            battle_style_name,
            battle_style_id: final_battle_style_id,
            days_display: self.days_parser.format_days(&local_day_of_weeks),
            quest_start_time: quest_start_time_local.format("%H:%M").to_string(),
            recruit_start_day_offset: recruit_day_offset,
            recruit_start_time: recruit_start_time_local.format("%H:%M").to_string(),
            note,
            timezone,
            dismissal_times,
        })
    }

    /// クエストを検索・取得
    async fn find_and_get_quest(
        &self,
        txn: &DatabaseTransaction,
        quest_alias: &str,
    ) -> Result<(i32, String, i32)> {
        let quest_repo = SeaOrmQuestRepository::new();

        // クエスト検索
        let search_results = quest_repo.search_by_name_or_alias(txn, quest_alias).await?;

        let quest_search_result = search_results.first().ok_or_else(|| {
            AppError::NotFound(format!("クエスト '{quest_alias}' が見つかりませんでした"))
        })?;

        let quest_id = quest_search_result.quest_id;

        // クエスト詳細取得
        let quest_detail = quest_repo
            .get_by_target_id(txn, quest_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "クエストID {quest_id} の詳細情報が見つかりませんでした"
                ))
            })?;

        Ok((
            quest_id,
            quest_detail.name.clone(),
            quest_detail.default_battle_style_id,
        ))
    }

    /// バトルスタイル名を取得
    async fn get_battle_style_name(
        &self,
        txn: &DatabaseTransaction,
        battle_style_id: i32,
    ) -> Result<String> {
        let battle_style_repo = SeaOrmBattleStyleRepository::new();
        let battle_style = battle_style_repo
            .get_by_id(txn, battle_style_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "バトルスタイルID {battle_style_id} が見つかりませんでした"
                ))
            })?;

        Ok(battle_style.display_name.clone())
    }

    /// マルチ募集チャンネルIDを取得
    async fn get_recruitment_channel(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<i64> {
        let guild_channel_repo = GuildChannelRepository::new();
        let guild_channel = guild_channel_repo
            .get_by_guild_and_type_with_txn(txn, guild_id, 2)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "マルチ募集チャンネルが登録されていません。\n\n\
                    定期募集を作成するには、先に管理者に `/チャンネル登録` コマンドで\
                    マルチ募集チャンネルを登録してもらってください。"
                    .to_string(),
            })?;

        Ok(guild_channel.channel_id)
    }

    /// 次回実行タスクをscheduled_tasksに登録
    ///
    /// 現在時刻から未来の次回実行日時を計算し、scheduled_tasksとscheduled_task_recurring_recruitmentsに登録
    /// 過去日時の場合は未来日時が見つかるまで繰り返し計算
    async fn create_next_scheduled_task(
        &self,
        txn: &DatabaseTransaction,
        schedule: &crate::models::entities::guild_master::battle_recruitment_schedules::Model,
        days: &[crate::models::entities::guild_master::battle_recruitment_schedule_days::Model],
    ) -> Result<()> {
        debug!(
            schedule_id = schedule.id,
            "次回実行タスクの作成を開始します"
        );

        let now = Utc::now();
        let mut search_from = now;
        let max_search_days = 365; // 最大1年先まで検索

        // 未来の次回実行日時が見つかるまでループ
        loop {
            let search_to = search_from + Duration::days(7);

            debug!(
                schedule_id = schedule.id,
                search_from = %search_from,
                search_to = %search_to,
                "次回実行日時を計算します"
            );

            // 次回募集日時を計算
            let next_times = self.schedule_service.calculate_next_recruitment_times(
                schedule,
                days,
                search_from,
                search_to,
            )?;

            // 最初に見つかった未来の募集開始日時を使用
            if let Some(next_time) = next_times.first() {
                if next_time.recruit_start_at > now {
                    // 未来日時が見つかった場合、scheduled_tasksに登録
                    let task_repo = ScheduledTaskRepository::new();
                    let task = task_repo
                        .create(
                            txn,
                            next_time.recruit_start_at,
                            ScheduledTaskType::RecurringRecruitment as i32,
                            Some(next_time.guild_id),
                            Some(next_time.channel_id),
                        )
                        .await?;

                    // scheduled_task_recurring_recruitmentsに関連付けを登録
                    let recurring_repo = ScheduledTaskRecurringRecruitmentRepository::new();
                    recurring_repo.create(txn, task.id, schedule.id).await?;

                    info!(
                        schedule_id = schedule.id,
                        task_id = task.id,
                        recruit_start_at = %next_time.recruit_start_at,
                        "次回実行タスクを登録しました"
                    );

                    return Ok(());
                }
            }

            // 次の検索範囲に進む
            search_from = search_to;

            // 無限ループ防止：最大検索日数を超えたらエラー
            if (search_from - now).num_days() > max_search_days {
                return Err(AppError::Business {
                    message: format!(
                        "次回実行日時が{}日以内に見つかりませんでした。スケジュール設定を確認してください。",
                        max_search_days
                    ),
                });
            }
        }
    }

    /// 解散時刻を保存
    async fn save_dismissal_times(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
        dismissal_times_str: &str,
        quest_start_time_local: chrono::NaiveTime,
        timezone: Tz,
    ) -> Result<()> {
        debug!(
            schedule_id,
            dismissal_times = %dismissal_times_str,
            "定期募集の解散時刻をパースします"
        );

        // 仮の出発日時を作成（パース用）
        let today = Utc::now().date_naive();
        let departure_time = timezone
            .from_local_datetime(&today.and_time(quest_start_time_local))
            .single()
            .ok_or_else(|| AppError::Business {
                message: "出発時刻の変換に失敗しました".to_string(),
            })?
            .with_timezone(&Utc);

        // 解散時刻をパース（統一パーサーを使用）
        let options = DateTimeParseOptions::for_dismissal_time(timezone, departure_time);
        let parsed_dismissal_times = parse_datetime(dismissal_times_str, &options)?;

        // データベースに保存
        let dismissal_repo = BattleRecruitmentScheduleDismissalRepository::new();

        // 元の入力値を分割（トリムして空文字除去）
        let input_values: Vec<&str> = dismissal_times_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for (idx, dismissal_time) in parsed_dismissal_times.iter().enumerate() {
            let input_value = input_values.get(idx).unwrap_or(&"").to_string();

            match dismissal_time {
                ParsedDateTime::Absolute(datetime) => {
                    // 絶対時刻の場合、時刻部分のみ抽出してTimeTimeに変換
                    let naive_time = datetime.time();
                    let dismissal_time = TimeTime::from_hms(
                        naive_time.hour() as u8,
                        naive_time.minute() as u8,
                        naive_time.second() as u8,
                    )
                    .map_err(|e| AppError::Business {
                        message: format!("解散時刻の変換に失敗しました: {}", e),
                    })?;
                    dismissal_repo
                        .create_absolute(txn, schedule_id, input_value, dismissal_time)
                        .await?;
                }
                ParsedDateTime::Relative {
                    days,
                    hours,
                    minutes,
                } => {
                    dismissal_repo
                        .create_relative(txn, schedule_id, input_value, *days, *hours, *minutes)
                        .await?;
                }
                ParsedDateTime::Time(_) => {
                    return Err(AppError::Business {
                        message: "解散時刻にTime型が返されました（想定外）".to_string(),
                    });
                }
            }
        }

        info!(
            schedule_id,
            count = dismissal_times_str.split(',').count(),
            "定期募集の解散時刻を保存しました"
        );

        Ok(())
    }
}

impl Default for ScheduleCreateService {
    fn default() -> Self {
        Self::new()
    }
}
