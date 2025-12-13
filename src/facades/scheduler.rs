use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::models::entities::{guild_channels, last_process_times::LastProcessType};
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::battle_style_repository::{BattleStyleRepository, SeaOrmBattleStyleRepository};
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::repository::database::last_process_time_repository::LastProcessTimeRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::{
    BattleRecruitmentScheduleRepository, NotificationRelBattleRecruitmentRepository,
    NotificationRelEventScheduleRepository, NotificationRepository, ScheduleRepository,
};
use crate::repository::quests_repository::QuestRepository;
use crate::services::recruitment::new::{create_initial_participants_text_for_buttons, create_recruitment_buttons};
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::services::schedule::{NotificationService, RecruitmentScheduleService, ScheduleCalculator};
use crate::services::timezone_service::TimezoneService;
use crate::types::{AppState, Result};
use chrono::{Duration, Utc};
use poise::serenity_prelude::{CreateEmbed, CreateMessage, Http};
use sea_orm::{DatabaseTransaction, EntityTrait, TransactionTrait};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// スケジューラーFacade
/// スケジュール管理の協調とトランザクション管理を担当
pub struct SchedulerFacade {
    app_state: Arc<AppState>,
}

impl SchedulerFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// スケジュールを生成
    /// イベントスケジュールと詳細から通知スケジュールを計算してDBに保存
    pub async fn generate_schedules(&self) -> Result<()> {
        info!("スケジュール生成を開始します");

        // スケジュール生成はSystemロールを使用（全ギルド対象）
        let txn = self.app_state.system_db().begin().await?;

        let result = async {
            let schedule_repo = ScheduleRepository::new();
            let notification_repo = NotificationRepository::new();
            let calculator = ScheduleCalculator::new();

            // 既存のスケジュールとリレーションをクリア
            debug!("既存のスケジュールを削除します");
            let rel_repo = NotificationRelEventScheduleRepository::new();
            rel_repo.delete_all_with_txn(&txn).await?;
            notification_repo.delete_all_with_txn(&txn).await?;

            // イベントスケジュールと詳細を取得
            let event_schedules = schedule_repo.find_all_event_schedules(self.app_state.system_db()).await?;
            let event_schedule_details = schedule_repo.find_all_event_schedule_details(self.app_state.system_db()).await?;

            debug!(
                event_schedules = event_schedules.len(),
                event_details = event_schedule_details.len(),
                "イベントスケジュールを取得しました"
            );

            if event_schedules.is_empty() {
                warn!("イベントスケジュールが登録されていません");
                return Ok(());
            }

            // 通知対象のギルドとチャンネルを取得（channel_type別）
            let guild_channels_by_type = self.get_notification_guild_channels_by_type().await?;

            debug!(
                channel_types = guild_channels_by_type.len(),
                "通知対象のギルド・チャンネルを取得しました"
            );

            if guild_channels_by_type.is_empty() {
                warn!("通知対象のギルド・チャンネルが登録されていません");
                return Ok(());
            }

            // スケジュールを計算
            let calculated_schedules = calculator.calculate_schedules(
                event_schedules,
                event_schedule_details,
                guild_channels_by_type,
            )?;

            debug!(
                calculated_schedules = calculated_schedules.len(),
                "スケジュールを計算しました"
            );

            // 計算されたスケジュールをDBに保存
            if !calculated_schedules.is_empty() {
                self.save_calculated_schedules(&txn, calculated_schedules)
                    .await?;
            }

            info!("スケジュール生成が完了しました");
            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!("スケジュール生成のトランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "スケジュール生成に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// 通知を実行
    /// last_process_timesから前回実行時刻を取得し、その時刻から現在までの通知を実行
    /// 各通知はis_sentフラグで管理されるため、last_process_timesの更新は不要
    pub async fn execute_notifications(&self, http: Arc<Http>) -> Result<()> {
        debug!("通知実行を開始します");

        let now = Utc::now();

        // 前回のスケジュール実行時刻を取得
        let last_process_time_repo = LastProcessTimeRepository::new();
        let last_process_time = last_process_time_repo
            .find_schedule_last_process_time(self.app_state.system_db())
            .await?;

        let last_execute_time = last_process_time.and_then(|lpt| lpt.execute_time);

        debug!(
            last_execute_time = ?last_execute_time,
            "前回のスケジュール実行時刻を取得しました"
        );

        // 通知を実行（各通知ごとにis_sentフラグを立てる）
        let notification_service = NotificationService::new(self.app_state.system_db().clone(), http.clone());
        notification_service
            .execute_scheduled_notifications(last_execute_time)
            .await?;

        // 定期募集を実行
        self.execute_recruitment_schedules(http.clone()).await?;

        // last_process_timesを更新（次回実行時の検索範囲を決定するため）
        let txn = self.app_state.system_db().begin().await?;

        let result = async {
            last_process_time_repo
                .upsert_with_txn(&txn, LastProcessType::Schedule, now)
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("last_process_timesを更新しました");
            }
            Err(e) => {
                error!(error = %e, "last_process_timesの更新に失敗しました");
                txn.rollback().await?;
                return Err(e);
            }
        }

        debug!("通知実行が完了しました");
        Ok(())
    }

    /// 通知対象のギルド・チャンネル一覧をchannel_type別に取得
    /// 戻り値: HashMap<channel_type, Vec<(guild_id, channel_id)>>
    async fn get_notification_guild_channels_by_type(&self) -> Result<HashMap<i32, Vec<(i64, i64)>>> {
        let guild_channels = guild_channels::Entity::find()
            .all(self.app_state.system_db())
            .await?;

        let mut channels_by_type: HashMap<i32, Vec<(i64, i64)>> = HashMap::new();

        for gc in guild_channels {
            channels_by_type
                .entry(gc.channel_type)
                .or_insert_with(Vec::new)
                .push((gc.guild_id, gc.channel_id));
        }

        debug!(
            channel_types = channels_by_type.len(),
            total_channels = channels_by_type.values().map(|v| v.len()).sum::<usize>(),
            "channel_type別のギルド・チャンネルを取得しました"
        );

        Ok(channels_by_type)
    }

    /// 計算されたスケジュールをDBに保存
    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelEventScheduleRepository::new();
        let now = Utc::now();

        debug!(count = schedules.len(), "通知とリレーションを作成します");

        let mut created_count = 0;
        let mut skipped_count = 0;

        // 各スケジュールに対して通知とリレーションを作成
        for schedule in schedules {
            // 通知日時が既に過ぎている場合はスキップ
            if schedule.schedule_datetime < now {
                debug!(
                    schedule_datetime = %schedule.schedule_datetime,
                    now = %now,
                    "通知日時が既に過ぎているためスキップします"
                );
                skipped_count += 1;
                continue;
            }

            // 通知を作成
            let notification = notification_repo
                .create_with_txn(
                    txn,
                    schedule.schedule_datetime,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
                .await?;

            // イベントスケジュールとのリレーションを作成
            rel_repo
                .create_with_txn(
                    txn,
                    schedule.event_schedule_id,
                    schedule.event_schedule_detail_id,
                    notification.id,
                )
                .await?;

            created_count += 1;
        }

        info!(
            created = created_count,
            skipped = skipped_count,
            "通知とリレーションの作成が完了しました"
        );
        Ok(())
    }

    /// 定期募集を実行
    /// 有効なスケジュールから募集を作成し、battle_recruitmentsに登録
    pub async fn execute_recruitment_schedules(&self, _http: Arc<Http>) -> Result<()> {
        debug!("定期募集実行を開始します");

        let now = Utc::now();

        // 前回の定期募集実行時刻を取得
        let last_process_time_repo = LastProcessTimeRepository::new();
        let last_process_time = last_process_time_repo
            .find_by_type(self.app_state.system_db(), LastProcessType::BattleRecruitmentSchedule)
            .await?;

        let last_execute_time = last_process_time.and_then(|lpt| lpt.execute_time);

        let from = last_execute_time.unwrap_or(now);

        debug!(
            last_execute_time = ?last_execute_time,
            from = %from,
            to = %now,
            "前回の定期募集実行時刻を取得しました"
        );

        // 有効な全スケジュールと曜日情報を取得
        let schedule_repo = BattleRecruitmentScheduleRepository::new();
        let schedules = schedule_repo
            .find_all_enabled_schedules_with_days(self.app_state.system_db())
            .await?;

        debug!(
            schedule_count = schedules.len(),
            "有効なスケジュールを取得しました"
        );

        if schedules.is_empty() {
            debug!("実行対象のスケジュールはありません");

            // last_process_timesを更新
            let txn = self.app_state.system_db().begin().await?;
            let result = async {
                last_process_time_repo
                    .upsert_with_txn(&txn, LastProcessType::BattleRecruitmentSchedule, now)
                    .await?;
                Ok::<(), crate::types::AppError>(())
            }
            .await;

            match result {
                Ok(_) => {
                    txn.commit().await?;
                    debug!("last_process_timesを更新しました");
                }
                Err(e) => {
                    error!(error = %e, "last_process_timesの更新に失敗しました");
                    txn.rollback().await?;
                    return Err(e);
                }
            }

            return Ok(());
        }

        // 各スケジュールについて募集日時を計算
        // DBの値は既にUTCなので、タイムゾーン取得・変換は不要
        let recruitment_service = RecruitmentScheduleService::new();
        let mut all_calculated_times = Vec::new();

        for (schedule, days) in &schedules {
            let calculated_times = recruitment_service
                .calculate_next_recruitment_times(schedule, days, from, now)?;

            debug!(
                schedule_id = schedule.id,
                calculated_count = calculated_times.len(),
                "募集日時を計算しました（UTC基準）"
            );

            all_calculated_times.extend(calculated_times);
        }

        info!(
            total_calculated = all_calculated_times.len(),
            "全スケジュールの募集日時計算が完了しました"
        );

        // 募集作成
        let mut created_count = 0;
        let mut skipped_count = 0;

        for calculated_time in all_calculated_times {
            // 募集作成処理を実行
            let create_result = self
                .create_recruitment_from_schedule(&_http, &calculated_time)
                .await;

            match create_result {
                Ok(_) => {
                    info!(
                        schedule_id = calculated_time.schedule_id,
                        quest_start_at = %calculated_time.quest_start_at,
                        "定期募集を作成しました"
                    );
                    created_count += 1;
                }
                Err(e) => {
                    error!(
                        error = %e,
                        schedule_id = calculated_time.schedule_id,
                        quest_start_at = %calculated_time.quest_start_at,
                        "定期募集の作成に失敗しました"
                    );
                    // エラーをログに記録するが、処理は継続する
                    skipped_count += 1;
                }
            }
        }

        info!(
            created = created_count,
            skipped = skipped_count,
            "定期募集の作成が完了しました"
        );

        // last_process_timesを更新
        let txn = self.app_state.system_db().begin().await?;

        let result = async {
            last_process_time_repo
                .upsert_with_txn(&txn, LastProcessType::BattleRecruitmentSchedule, now)
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("last_process_timesを更新しました");
            }
            Err(e) => {
                error!(error = %e, "last_process_timesの更新に失敗しました");
                txn.rollback().await?;
                return Err(e);
            }
        }

        debug!("定期募集実行が完了しました");
        Ok(())
    }

    /// スケジュールから募集を作成
    /// 定期募集スケジュールに基づいて実際の募集を作成
    async fn create_recruitment_from_schedule(
        &self,
        http: &Arc<Http>,
        calculated_time: &crate::services::schedule::CalculatedRecruitmentTime,
    ) -> Result<()> {
        debug!(
            schedule_id = calculated_time.schedule_id,
            quest_id = calculated_time.quest_id,
            "スケジュールから募集を作成します"
        );

        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, calculated_time.guild_id).await?;

        let result = async {
            // 0. マルチ募集チャンネルを取得（channel_type = 1）
            let guild_channel_repo = GuildChannelRepository::new();
            let guild_channel = guild_channel_repo
                // channel_type = 2 (マルチ募集チャンネル)
                .get_by_guild_and_type_with_txn(&txn, calculated_time.guild_id, 2)
                .await?
                .ok_or_else(|| crate::types::AppError::NotFound(format!(
                    "ギルドID {} にマルチ募集チャンネルが登録されていません",
                    calculated_time.guild_id
                )))?;

            let recruitment_channel_id = guild_channel.channel_id;
            debug!(
                recruitment_channel_id = recruitment_channel_id,
                "マルチ募集チャンネルを取得しました"
            );

            // 1. Quest, BattleStyle, タイムゾーンを取得
            let quest_repo = SeaOrmQuestRepository::new();
            let battle_style_repo = SeaOrmBattleStyleRepository::new();
            let timezone_repo = Arc::new(GuildTimezoneRepository::new());
            let timezone_service = TimezoneService::new(timezone_repo);

            let quest = quest_repo
                .get_by_target_id(conn, calculated_time.quest_id)
                .await?
                .ok_or_else(|| crate::types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    calculated_time.quest_id
                )))?;

            let battle_style = battle_style_repo
                .get_by_id(conn, calculated_time.battle_style_id)
                .await?
                .ok_or_else(|| crate::types::AppError::NotFound(format!(
                    "攻略方法ID {} が見つかりませんでした",
                    calculated_time.battle_style_id
                )))?;

            let timezone = timezone_service
                .get_guild_timezone(conn, calculated_time.guild_id)
                .await?;

            // 2. ロールメンションを取得
            let role_service = RoleNotificationService::new();
            let role_mentions = role_service
                .get_role_mentions(&txn, calculated_time.guild_id, quest.id)
                .await?;

            // 3. メッセージ内容を作成
            let mut message_content = crate::services::recruitment::new::create_message_content(
                &quest.name,
                &battle_style.display_name,
                &calculated_time.quest_start_at,
                timezone,
            );

            // 備考がある場合は追加
            if let Some(note) = &calculated_time.note {
                message_content.push_str(&format!("\n備考: {}", note));
            }

            // ロールメンションを先頭に追加
            if !role_mentions.is_empty() {
                debug!(role_mentions = %role_mentions, "ロールメンションを募集メッセージの先頭に追加します");
                message_content = format!("{}\n{}", role_mentions, message_content);
            }

            // 4. Embedを作成
            let initial_participants_text =
                create_initial_participants_text_for_buttons(&battle_style.display_name);
            let embed = CreateEmbed::new()
                .title("参加者一覧")
                .description(&initial_participants_text)
                .color(0x0099ff);

            // 5. ボタンを作成
            let buttons = create_recruitment_buttons(&battle_style.display_name);

            // 6. Discordメッセージを投稿（マルチ募集チャンネルに投稿）
            let channel_id = poise::serenity_prelude::ChannelId::new(recruitment_channel_id as u64);
            let message = channel_id
                .send_message(
                    http,
                    CreateMessage::new()
                        .content(message_content)
                        .embed(embed)
                        .components(buttons),
                )
                .await?;

            let message_id = message.id.get();

            debug!(message_id = %message_id, "Discordメッセージを投稿しました");

            // 7. battle_recruitmentsに保存
            let repos = RepositoryContainer::new();
            let battle_recruitment_repo = repos.battle_recruitment();

            let recruitment = battle_recruitment_repo
                .create_with_txn(
                    &txn,
                    calculated_time.guild_id as u64,
                    recruitment_channel_id as u64,
                    message_id,
                    quest.id,
                    calculated_time.battle_style_id,
                    calculated_time.quest_start_at,
                )
                .await?;

            info!(
                recruitment_id = recruitment.id,
                "募集をデータベースに登録しました"
            );

            // 8. 出発時刻の通知を登録（出発5分前）
            let notification_repo = NotificationRepository::new();
            let notify_time = calculated_time.quest_start_at - Duration::minutes(5);

            debug!(
                quest_start_at = %calculated_time.quest_start_at,
                notify_time = %notify_time,
                "募集の出発通知を登録します"
            );

            let notification = notification_repo
                .create_with_txn(
                    &txn,
                    notify_time,
                    calculated_time.guild_id,
                    recruitment_channel_id,
                    "MSG00033".to_string(),
                )
                .await?;

            info!("募集の出発通知を登録しました");

            // 9. 通知と募集のリレーションを作成
            let rel_repo = NotificationRelBattleRecruitmentRepository::new();
            rel_repo
                .create_with_txn(&txn, recruitment.id, notification.id)
                .await?;

            info!("募集と通知のリレーションを登録しました");

            Ok::<(), crate::types::AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                debug!("募集作成トランザクションをコミットしました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "募集作成トランザクションをロールバックしました");
                Err(e)
            }
        }
    }
}
