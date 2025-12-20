use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::repository::quests_repository::QuestRepository;
use crate::services::recruitment::schedule::{DaysParserService, TimeParserService};
use crate::services::schedule::{RecruitmentScheduleService, convert_local_days_and_time_to_utc};
use crate::types::{AppError, Result};
use chrono_tz::Tz;
use sea_orm::DatabaseTransaction;
use tracing::info;

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
}

/// スケジュール作成サービス
///
/// 定期募集スケジュールの作成ビジネスロジックを担当するサービス。
pub struct ScheduleCreateService {
    time_parser: TimeParserService,
    days_parser: DaysParserService,
    schedule_service: RecruitmentScheduleService,
}

impl ScheduleCreateService {
    pub fn new() -> Self {
        Self {
            time_parser: TimeParserService::new(),
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
        let quest_start_time_local = self.time_parser.parse_time_string(quest_start_time)?;
        let recruit_start_time_local = self.time_parser.parse_time_string(recruit_start_time)?;
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
        let (schedule, _) = schedule_repo
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

        // 9. 結果データ作成
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
}

impl Default for ScheduleCreateService {
    fn default() -> Self {
        Self::new()
    }
}
