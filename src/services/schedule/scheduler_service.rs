use crate::models::entities::guild_master::{
    guild_channels, guild_event_schedule_details, guild_event_schedules,
};
use crate::models::entities::master::{event_schedule_details, event_schedules};
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::services::schedule::ScheduleCalculator;
use crate::services::schedule::schedule_calculator::CalculatedSchedule;
use crate::types::{AppState, Result};
use sea_orm::DatabaseTransaction;
use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::{debug, info, warn};

use crate::models::entities::guild_master::battle_recruitment_schedules;
use crate::models::entities::worker::last_process_times::LastProcessType;
use crate::models::last_process_times::LastProcessTime;
use crate::repository::LastProcessTimeRepository;
use crate::repository::schedule::{
    BattleRecruitmentScheduleRepository, NotificationRelEventScheduleRepository,
    NotificationRepository, ScheduleRepository, ScheduledTaskRepository,
};
use sea_orm::DatabaseConnection;

#[derive(Debug, Default, Clone, Copy)]
struct NotificationSchedulePlanStats {
    target_guilds: usize,
    planned: usize,
    skipped_channel_unresolved: usize,
    skipped_datetime_calc_failed: usize,
}

fn resolve_target_task_types(task_type: Option<ScheduledTaskType>) -> Vec<ScheduledTaskType> {
    match task_type {
        Some(task_type) => vec![task_type],
        None => all_task_types().into(),
    }
}

fn all_task_types() -> [ScheduledTaskType; 7] {
    [
        ScheduledTaskType::Notification,
        ScheduledTaskType::Dissolution,
        ScheduledTaskType::DataCleanup,
        ScheduledTaskType::RecurringRecruitment,
        ScheduledTaskType::Dismissal,
        ScheduledTaskType::AutoRecruitmentRotation,
        ScheduledTaskType::AutoMatching,
    ]
}

fn plan_notification_schedules(
    calculator: &ScheduleCalculator,
    global_event_schedules: Vec<event_schedules::Model>,
    global_event_schedule_details: Vec<event_schedule_details::Model>,
    guild_event_schedules: Vec<guild_event_schedules::Model>,
    guild_event_schedule_details: Vec<guild_event_schedule_details::Model>,
    guild_channels_by_guild: HashMap<i64, HashMap<i32, i64>>,
) -> Result<(Vec<CalculatedSchedule>, NotificationSchedulePlanStats)> {
    let mut stats = NotificationSchedulePlanStats::default();

    let global_schedule_ids: HashSet<uuid::Uuid> =
        global_event_schedules.iter().map(|s| s.id).collect();
    let global_detail_ids: HashSet<uuid::Uuid> =
        global_event_schedule_details.iter().map(|d| d.id).collect();

    let mut global_schedules_by_id: HashMap<uuid::Uuid, event_schedules::Model> = HashMap::new();
    for schedule in global_event_schedules {
        global_schedules_by_id.insert(schedule.id, schedule);
    }

    let mut global_details_by_profile: HashMap<
        String,
        HashMap<uuid::Uuid, event_schedule_details::Model>,
    > = HashMap::new();
    for detail in global_event_schedule_details {
        global_details_by_profile
            .entry(detail.profile.clone())
            .or_default()
            .insert(detail.id, detail);
    }

    let mut guild_schedules_by_guild: HashMap<
        i64,
        HashMap<uuid::Uuid, guild_event_schedules::Model>,
    > = HashMap::new();
    for schedule in guild_event_schedules {
        guild_schedules_by_guild
            .entry(schedule.guild_id)
            .or_default()
            .insert(schedule.id, schedule);
    }

    let mut guild_details_by_guild_profile: HashMap<
        i64,
        HashMap<String, HashMap<uuid::Uuid, guild_event_schedule_details::Model>>,
    > = HashMap::new();
    for detail in guild_event_schedule_details {
        guild_details_by_guild_profile
            .entry(detail.guild_id)
            .or_default()
            .entry(detail.profile.clone())
            .or_default()
            .insert(detail.id, detail);
    }

    let mut target_guild_ids = BTreeSet::new();
    target_guild_ids.extend(guild_channels_by_guild.keys().copied());
    target_guild_ids.extend(guild_schedules_by_guild.keys().copied());
    target_guild_ids.extend(guild_details_by_guild_profile.keys().copied());

    stats.target_guilds = target_guild_ids.len();

    let mut results = Vec::new();

    for guild_id in target_guild_ids {
        let guild_channels = guild_channels_by_guild.get(&guild_id);
        let guild_schedules = guild_schedules_by_guild.get(&guild_id);
        let guild_details = guild_details_by_guild_profile.get(&guild_id);

        // globalスケジュール + guild上書き + guild独自（global未存在）を統合
        let mut effective_schedules: Vec<(event_schedules::Model, Option<uuid::Uuid>)> = Vec::new();

        for (schedule_id, global_schedule) in &global_schedules_by_id {
            if let Some(guild_schedule) = guild_schedules.and_then(|m| m.get(schedule_id)) {
                effective_schedules
                    .push((to_master_event_schedule(guild_schedule), Some(*schedule_id)));
            } else {
                effective_schedules.push((global_schedule.clone(), Some(*schedule_id)));
            }
        }

        if let Some(guild_schedules) = guild_schedules {
            for (schedule_id, guild_schedule) in guild_schedules {
                if global_schedule_ids.contains(schedule_id) {
                    continue;
                }
                // master.event_schedulesに存在しないIDは、リレーション保存をスキップする
                effective_schedules.push((to_master_event_schedule(guild_schedule), None));
            }
        }

        for (event_schedule, relation_event_schedule_id) in effective_schedules {
            let global_details_for_profile = global_details_by_profile.get(&event_schedule.profile);
            let guild_details_for_profile =
                guild_details.and_then(|m| m.get(&event_schedule.profile));

            let mut detail_ids = BTreeSet::new();
            if let Some(global_details) = global_details_for_profile {
                detail_ids.extend(global_details.keys().copied());
            }
            if let Some(guild_details) = guild_details_for_profile {
                detail_ids.extend(guild_details.keys().copied());
            }

            if detail_ids.is_empty() {
                debug!(
                    guild_id = guild_id,
                    profile = %event_schedule.profile,
                    "該当プロファイルの詳細スケジュールが存在しないためスキップします"
                );
                continue;
            }

            for detail_id in detail_ids {
                let global_detail = global_details_for_profile.and_then(|m| m.get(&detail_id));
                let guild_detail = guild_details_for_profile.and_then(|m| m.get(&detail_id));

                let message_text_id = if let Some(guild_detail) = guild_detail {
                    guild_detail.message_text_id.clone()
                } else if let Some(global_detail) = global_detail {
                    global_detail.message_text_id.clone()
                } else {
                    // detail_idsに含めた時点で必ず存在するはず
                    stats.skipped_datetime_calc_failed += 1;
                    warn!(
                        guild_id = guild_id,
                        profile = %event_schedule.profile,
                        detail_id = %detail_id,
                        "詳細スケジュールが解決できませんでした"
                    );
                    continue;
                };

                let channel_id =
                    resolve_channel_id(guild_id, guild_channels, guild_detail, global_detail);

                let Some(channel_id) = channel_id else {
                    stats.skipped_channel_unresolved += 1;
                    warn!(
                        guild_id = guild_id,
                        profile = %event_schedule.profile,
                        detail_id = %detail_id,
                        "通知先チャンネルを解決できないためスキップします"
                    );
                    continue;
                };

                let detail_for_calc = if let Some(guild_detail) = guild_detail {
                    to_master_event_schedule_detail(guild_detail)
                } else if let Some(global_detail) = global_detail {
                    global_detail.clone()
                } else {
                    stats.skipped_datetime_calc_failed += 1;
                    warn!(
                        guild_id = guild_id,
                        profile = %event_schedule.profile,
                        detail_id = %detail_id,
                        "詳細スケジュールが解決できませんでした"
                    );
                    continue;
                };

                match calculator.calculate_datetimes(&event_schedule, &detail_for_calc) {
                    Ok(schedule_datetimes) => {
                        for schedule_datetime in schedule_datetimes {
                            results.push(CalculatedSchedule {
                                schedule_datetime,
                                guild_id,
                                channel_id,
                                message_text_id: message_text_id.clone(),
                                event_schedule_id: relation_event_schedule_id,
                                event_schedule_detail_id: global_detail_ids
                                    .contains(&detail_id)
                                    .then_some(detail_id),
                            });
                            stats.planned += 1;
                        }
                    }
                    Err(e) => {
                        stats.skipped_datetime_calc_failed += 1;
                        warn!(
                            error = %e,
                            guild_id = guild_id,
                            profile = %event_schedule.profile,
                            detail_id = %detail_id,
                            "スケジュール日時の計算に失敗しました"
                        );
                    }
                }
            }
        }
    }

    Ok((results, stats))
}

fn resolve_channel_id(
    guild_id: i64,
    guild_channels: Option<&HashMap<i32, i64>>,
    guild_detail: Option<&guild_event_schedule_details::Model>,
    global_detail: Option<&event_schedule_details::Model>,
) -> Option<i64> {
    let guild_channels = guild_channels?;

    if let Some(detail) = guild_detail {
        if let Some(channel_id) = detail.notification_channel_id {
            return Some(channel_id);
        }

        if let Some(channel_id) = guild_channels.get(&detail.notification_channel_type) {
            return Some(*channel_id);
        }
    }

    if let Some(detail) = global_detail
        && let Some(channel_id) = guild_channels.get(&detail.notification_channel_type)
    {
        return Some(*channel_id);
    }

    debug!(
        guild_id = guild_id,
        guild_channel_types = guild_channels.len(),
        guild_detail_channel_id = guild_detail.and_then(|d| d.notification_channel_id),
        guild_detail_channel_type = guild_detail.map(|d| d.notification_channel_type),
        global_detail_channel_type = global_detail.map(|d| d.notification_channel_type),
        "通知先チャンネルが未登録のため解決できませんでした"
    );

    None
}

fn to_master_event_schedule(schedule: &guild_event_schedules::Model) -> event_schedules::Model {
    event_schedules::Model {
        id: schedule.id,
        event_type: schedule.event_type.clone(),
        event_count: schedule.event_count,
        profile: schedule.profile.clone(),
        weak_attribute: schedule.weak_attribute,
        start_at: schedule.start_at,
        end_at: schedule.end_at,
        created_at: schedule.created_at,
        updated_at: schedule.updated_at,
    }
}

fn to_master_event_schedule_detail(
    detail: &guild_event_schedule_details::Model,
) -> event_schedule_details::Model {
    event_schedule_details::Model {
        id: detail.id,
        profile: detail.profile.clone(),
        start_day_relative: detail.start_day_relative.clone(),
        time: detail.time.clone(),
        schedule_name: detail.schedule_name.clone(),
        message_text_id: detail.message_text_id.clone(),
        notification_channel_type: detail.notification_channel_type,
        reactions: detail.reactions.clone(),
        created_at: detail.created_at,
        updated_at: detail.updated_at,
    }
}

/// スケジューラーサービス
pub struct SchedulerService<SR, NR, NRER, STR, BRSR, LPTR>
where
    SR: ScheduleRepository,
    NR: NotificationRepository,
    NRER: NotificationRelEventScheduleRepository,
    STR: ScheduledTaskRepository,
    BRSR: BattleRecruitmentScheduleRepository,
    LPTR: LastProcessTimeRepository,
{
    schedule_repo: SR,
    notification_repo: NR,
    rel_repo: NRER,
    scheduled_task_repo: STR,
    battle_recruitment_schedule_repo: BRSR,
    last_process_time_repo: LPTR,
}

impl<SR, NR, NRER, STR, BRSR, LPTR> SchedulerService<SR, NR, NRER, STR, BRSR, LPTR>
where
    SR: ScheduleRepository,
    NR: NotificationRepository,
    NRER: NotificationRelEventScheduleRepository,
    STR: ScheduledTaskRepository,
    BRSR: BattleRecruitmentScheduleRepository,
    LPTR: LastProcessTimeRepository,
{
    pub fn new(
        schedule_repo: SR,
        notification_repo: NR,
        rel_repo: NRER,
        scheduled_task_repo: STR,
        battle_recruitment_schedule_repo: BRSR,
        last_process_time_repo: LPTR,
    ) -> Self {
        Self {
            schedule_repo,
            notification_repo,
            rel_repo,
            scheduled_task_repo,
            battle_recruitment_schedule_repo,
            last_process_time_repo,
        }
    }

    /// ギルド向けにイベントスケジュールから通知スケジュールを計算し保存する
    /// - トランザクション境界はFacadeが管理
    pub async fn generate_and_persist_schedules_for_guild(
        &self,
        txn: &DatabaseTransaction,
        app_state: &AppState,
        guild_id: i64,
        task_type: Option<ScheduledTaskType>,
    ) -> Result<()> {
        self.generate_and_persist_schedules_internal(txn, app_state, Some(guild_id), task_type)
            .await
    }

    /// 管理サーバー向けにイベントスケジュールから通知スケジュールを計算し保存する
    /// - トランザクション境界はFacadeが管理
    pub async fn generate_and_persist_schedules_for_global(
        &self,
        txn: &DatabaseTransaction,
        app_state: &AppState,
        task_type: Option<ScheduledTaskType>,
    ) -> Result<()> {
        self.generate_and_persist_schedules_internal(txn, app_state, None, task_type)
            .await
    }

    /// イベントスケジュールから通知スケジュールを計算し保存する
    /// 既存呼び出しとの互換性のため、管理サーバー向け全体再生成に委譲
    pub async fn generate_and_persist_schedules(
        &self,
        txn: &DatabaseTransaction,
        app_state: &AppState,
    ) -> Result<()> {
        self.generate_and_persist_schedules_for_global(txn, app_state, None)
            .await
    }

    async fn generate_and_persist_schedules_internal(
        &self,
        txn: &DatabaseTransaction,
        app_state: &AppState,
        target_guild_id: Option<i64>,
        task_type: Option<ScheduledTaskType>,
    ) -> Result<()> {
        let target_task_types = resolve_target_task_types(task_type);

        if task_type.is_some() {
            for unsupported_task_type in target_task_types
                .iter()
                .copied()
                .filter(|t| *t != ScheduledTaskType::Notification)
            {
                warn!(
                    task_type = unsupported_task_type.as_i32(),
                    description = unsupported_task_type.description(),
                    "指定されたタスク種別の再生成は未実装のためスキップします"
                );
            }
        }

        if !target_task_types.contains(&ScheduledTaskType::Notification) {
            let task_type_label = task_type.map(|value| value.description()).unwrap_or("不明");
            return Err(crate::types::AppError::Business {
                message: format!("指定されたタスク種別（{task_type_label}）の再生成は未実装です"),
            });
        }

        let calculator = ScheduleCalculator::new(app_state.config.max_schedule_days_outside_event);

        // 既存の通知スケジュールをクリア（CASCADEで関連通知・リレーションも削除）
        let deleted_tasks = match target_guild_id {
            Some(guild_id) => {
                self.scheduled_task_repo
                    .delete_all_by_task_type_and_guild(
                        txn,
                        ScheduledTaskType::Notification.as_i32(),
                        guild_id,
                    )
                    .await?
            }
            None => {
                self.scheduled_task_repo
                    .delete_all_by_task_type(txn, ScheduledTaskType::Notification.as_i32())
                    .await?
            }
        };

        debug!(
            target_guild_id,
            deleted_tasks = deleted_tasks,
            "通知タイプのscheduled_tasksとnotificationsを削除しました"
        );

        // イベントスケジュール（global/guild）と詳細（global/guild）を取得
        let global_event_schedules = self.schedule_repo.find_all_event_schedules(txn).await?;
        let global_event_schedule_details = self
            .schedule_repo
            .find_all_event_schedule_details(txn)
            .await?;
        let mut guild_event_schedules = self
            .schedule_repo
            .find_all_guild_event_schedules(txn)
            .await?;
        let mut guild_event_schedule_details = self
            .schedule_repo
            .find_all_guild_event_schedule_details(txn)
            .await?;

        if let Some(guild_id) = target_guild_id {
            guild_event_schedules.retain(|model| model.guild_id == guild_id);
            guild_event_schedule_details.retain(|model| model.guild_id == guild_id);
        }

        debug!(
            target_guild_id,
            global_event_schedules = global_event_schedules.len(),
            global_event_details = global_event_schedule_details.len(),
            guild_event_schedules = guild_event_schedules.len(),
            guild_event_details = guild_event_schedule_details.len(),
            "イベントスケジュール（global/guild）を取得しました"
        );

        if global_event_schedules.is_empty() && guild_event_schedules.is_empty() {
            warn!("イベントスケジュール（global/guild）が登録されていません");
            return Ok(());
        }

        // 通知対象のギルド・チャンネルを取得（guild_id -> channel_type -> channel_id）
        let mut guild_channels_by_guild =
            self.get_notification_guild_channels_by_guild(txn).await?;

        if let Some(guild_id) = target_guild_id {
            guild_channels_by_guild.retain(|key, _| *key == guild_id);
            guild_channels_by_guild.entry(guild_id).or_default();
        }

        debug!(
            target_guild_id,
            guilds = guild_channels_by_guild.len(),
            "通知対象のギルド・チャンネルを取得しました（guild単位）"
        );

        if guild_channels_by_guild.is_empty() {
            warn!("通知対象のギルド・チャンネルが登録されていません");
            return Ok(());
        }

        // スケジュールを計算（global/guild統合）
        let (calculated_schedules, plan_stats) = plan_notification_schedules(
            &calculator,
            global_event_schedules,
            global_event_schedule_details,
            guild_event_schedules,
            guild_event_schedule_details,
            guild_channels_by_guild,
        )?;

        debug!(
            calculated_schedules = calculated_schedules.len(),
            "スケジュールを計算しました"
        );

        debug!(
            target_guilds = plan_stats.target_guilds,
            planned = plan_stats.planned,
            skipped_channel_unresolved = plan_stats.skipped_channel_unresolved,
            skipped_datetime_calc_failed = plan_stats.skipped_datetime_calc_failed,
            "通知スケジュールの統合計画が完了しました"
        );

        // 計算されたスケジュールをDBに保存
        if !calculated_schedules.is_empty() {
            self.save_calculated_schedules(txn, calculated_schedules)
                .await?;
        }

        info!(target_guild_id, "スケジュール生成が完了しました");
        Ok(())
    }

    async fn get_notification_guild_channels_by_guild<C>(
        &self,
        conn: &C,
    ) -> Result<HashMap<i64, HashMap<i32, i64>>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let guild_channels = <guild_channels::Entity as sea_orm::EntityTrait>::find()
            .all(conn)
            .await?;

        let mut channels_by_guild: HashMap<i64, HashMap<i32, i64>> = HashMap::new();

        for gc in guild_channels {
            channels_by_guild
                .entry(gc.guild_id)
                .or_default()
                .insert(gc.channel_type, gc.channel_id);
        }
        Ok(channels_by_guild)
    }

    async fn save_calculated_schedules(
        &self,
        txn: &DatabaseTransaction,
        schedules: Vec<CalculatedSchedule>,
    ) -> Result<()> {
        use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
        use chrono::Utc;

        let now = Utc::now();

        let mut created_count = 0;
        let mut skipped_count = 0;

        for schedule in schedules {
            if schedule.schedule_datetime < now {
                skipped_count += 1;
                continue;
            }

            // 1. scheduled_taskを作成（task_type=1: Notification）
            let scheduled_task = self
                .scheduled_task_repo
                .create(
                    txn,
                    schedule.schedule_datetime,
                    ScheduledTaskType::Notification.as_i32(),
                    Some(schedule.guild_id),
                    Some(schedule.channel_id),
                )
                .await?;

            // 2. notificationを作成（task_idを指定）
            let notification = self
                .notification_repo
                .create_with_txn(
                    txn,
                    scheduled_task.id,
                    schedule.guild_id,
                    schedule.channel_id,
                    schedule.message_text_id,
                )
                .await?;

            // 3. notification_relを作成
            if let Some(event_schedule_id) = schedule.event_schedule_id {
                self.rel_repo
                    .create_with_txn(
                        txn,
                        event_schedule_id,
                        schedule.event_schedule_detail_id,
                        notification.id,
                    )
                    .await?;
            }

            created_count += 1;
        }

        debug!(
            created = created_count,
            skipped = skipped_count,
            "通知とリレーションの作成が完了しました"
        );
        Ok(())
    }

    /// 前回の処理時刻を取得
    ///
    /// # 注意
    /// 通知処理用のlast_process_timesは廃止予定です。
    /// SchedulerManagerを使用してください。
    /// 定期募集処理では引き続き使用されます。
    pub async fn get_last_process_time(
        &self,
        db: &DatabaseConnection,
        process_type: LastProcessType,
    ) -> Result<Option<LastProcessTime>> {
        match process_type {
            LastProcessType::Schedule => {
                self.last_process_time_repo
                    .find_schedule_last_process_time(db)
                    .await
            }
            _ => {
                self.last_process_time_repo
                    .find_by_type(db, process_type)
                    .await
            }
        }
    }

    /// 処理時刻を更新
    ///
    /// # 注意
    /// 通知処理用のlast_process_timesは廃止予定です。
    /// SchedulerManagerを使用してください。
    /// 定期募集処理では引き続き使用されます。
    pub async fn update_last_process_time(
        &self,
        txn: &DatabaseTransaction,
        process_type: LastProcessType,
        execute_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.last_process_time_repo
            .upsert_with_txn(txn, process_type, execute_time)
            .await
            .map(|_| ())
    }

    /// 有効な募集スケジュールを曜日情報付きで取得
    pub async fn find_enabled_recruitment_schedules_with_days(
        &self,
        db: &DatabaseConnection,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<crate::models::entities::guild_master::battle_recruitment_schedule_days::Model>,
        )>,
    > {
        self.battle_recruitment_schedule_repo
            .find_all_enabled_schedules_with_days(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    fn build_global_event_schedule(id: Uuid, profile: &str) -> event_schedules::Model {
        event_schedules::Model {
            id,
            event_type: "gw".to_string(),
            event_count: 1,
            profile: profile.to_string(),
            weak_attribute: 1,
            start_at: NaiveDate::from_ymd_opt(2025, 1, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            end_at: NaiveDate::from_ymd_opt(2025, 1, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_guild_detail(
        guild_id: i64,
        id: Uuid,
        profile: &str,
        message_text_id: &str,
        notification_channel_type: i32,
        notification_channel_id: Option<i64>,
    ) -> guild_event_schedule_details::Model {
        guild_event_schedule_details::Model {
            guild_id,
            id,
            profile: profile.to_string(),
            start_day_relative: "0".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "guild_detail".to_string(),
            message_text_id: message_text_id.to_string(),
            notification_channel_type,
            notification_channel_id,
            reactions: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_global_detail(
        id: Uuid,
        profile: &str,
        message_text_id: &str,
        notification_channel_type: i32,
    ) -> event_schedule_details::Model {
        event_schedule_details::Model {
            id,
            profile: profile.to_string(),
            start_day_relative: "0".to_string(),
            time: "05:00:00".to_string(),
            schedule_name: "global_detail".to_string(),
            message_text_id: message_text_id.to_string(),
            notification_channel_type,
            reactions: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn guild_detail_only_with_global_schedule_uses_global_message_id_pattern_153() {
        let calculator = ScheduleCalculator::new(365);
        let guild_id = 1001_i64;
        let other_guild_id = 2002_i64;
        let schedule_id = Uuid::new_v4();
        let detail_id = Uuid::new_v4();

        let mut guild_channels_by_guild = HashMap::new();
        guild_channels_by_guild.insert(guild_id, HashMap::from([(10, 5001)]));
        guild_channels_by_guild.insert(other_guild_id, HashMap::from([(10, 6001)]));

        let (results, stats) = plan_notification_schedules(
            &calculator,
            vec![build_global_event_schedule(schedule_id, "gw_profile")],
            vec![],
            vec![],
            vec![build_guild_detail(
                guild_id,
                detail_id,
                "gw_profile",
                "message_global_only",
                10,
                None,
            )],
            guild_channels_by_guild,
        )
        .unwrap();

        assert_eq!(stats.target_guilds, 2);
        assert_eq!(stats.planned, 1);
        assert_eq!(results.len(), 1);

        let actual = &results[0];
        assert_eq!(actual.guild_id, guild_id);
        assert_eq!(actual.channel_id, 5001);
        assert_eq!(actual.message_text_id, "message_global_only");
        assert_eq!(actual.event_schedule_id, Some(schedule_id));
        assert_eq!(actual.event_schedule_detail_id, None);
    }

    #[test]
    fn guild_detail_only_with_global_schedule_uses_guild_message_id_pattern_156() {
        let calculator = ScheduleCalculator::new(365);
        let guild_id = 3003_i64;
        let schedule_id = Uuid::new_v4();
        let detail_id = Uuid::new_v4();

        let mut guild_channels_by_guild = HashMap::new();
        guild_channels_by_guild.insert(guild_id, HashMap::from([(10, 7001)]));

        let (results, stats) = plan_notification_schedules(
            &calculator,
            vec![build_global_event_schedule(schedule_id, "gw_profile")],
            vec![],
            vec![],
            vec![build_guild_detail(
                guild_id,
                detail_id,
                "gw_profile",
                "message_guild_only",
                10,
                None,
            )],
            guild_channels_by_guild,
        )
        .unwrap();

        assert_eq!(stats.target_guilds, 1);
        assert_eq!(stats.planned, 1);
        assert_eq!(results.len(), 1);

        let actual = &results[0];
        assert_eq!(actual.guild_id, guild_id);
        assert_eq!(actual.channel_id, 7001);
        assert_eq!(actual.message_text_id, "message_guild_only");
        assert_eq!(actual.event_schedule_id, Some(schedule_id));
        assert_eq!(actual.event_schedule_detail_id, None);
    }

    #[test]
    fn resolve_channel_id_prefers_guild_notification_channel_id_over_channel_type() {
        let calculator = ScheduleCalculator::new(365);
        let guild_id = 4004_i64;
        let schedule_id = Uuid::new_v4();
        let detail_id = Uuid::new_v4();

        let mut guild_channels_by_guild = HashMap::new();
        guild_channels_by_guild.insert(guild_id, HashMap::from([(10, 7101)]));

        let (results, stats) = plan_notification_schedules(
            &calculator,
            vec![build_global_event_schedule(schedule_id, "gw_profile")],
            vec![build_global_detail(
                detail_id,
                "gw_profile",
                "message_global",
                99,
            )],
            vec![],
            vec![build_guild_detail(
                guild_id,
                detail_id,
                "gw_profile",
                "message_guild",
                10,
                Some(8101),
            )],
            guild_channels_by_guild,
        )
        .unwrap();

        assert_eq!(stats.target_guilds, 1);
        assert_eq!(stats.planned, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].channel_id, 8101);
    }

    #[test]
    fn resolve_channel_id_falls_back_to_global_notification_channel_type() {
        let calculator = ScheduleCalculator::new(365);
        let guild_id = 5005_i64;
        let schedule_id = Uuid::new_v4();
        let detail_id = Uuid::new_v4();

        let mut guild_channels_by_guild = HashMap::new();
        guild_channels_by_guild.insert(guild_id, HashMap::from([(10, 7201)]));

        let (results, stats) = plan_notification_schedules(
            &calculator,
            vec![build_global_event_schedule(schedule_id, "gw_profile")],
            vec![build_global_detail(
                detail_id,
                "gw_profile",
                "message_global",
                10,
            )],
            vec![],
            vec![build_guild_detail(
                guild_id,
                detail_id,
                "gw_profile",
                "message_guild",
                88,
                None,
            )],
            guild_channels_by_guild,
        )
        .unwrap();

        assert_eq!(stats.target_guilds, 1);
        assert_eq!(stats.planned, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].channel_id, 7201);
    }

    #[test]
    fn resolve_channel_id_fails_when_global_channel_type_is_unresolvable() {
        let calculator = ScheduleCalculator::new(365);
        let guild_id = 6006_i64;
        let schedule_id = Uuid::new_v4();
        let detail_id = Uuid::new_v4();

        let mut guild_channels_by_guild = HashMap::new();
        guild_channels_by_guild.insert(guild_id, HashMap::from([(10, 7301)]));

        let (results, stats) = plan_notification_schedules(
            &calculator,
            vec![build_global_event_schedule(schedule_id, "gw_profile")],
            vec![build_global_detail(
                detail_id,
                "gw_profile",
                "message_global",
                99,
            )],
            vec![],
            vec![],
            guild_channels_by_guild,
        )
        .unwrap();

        assert_eq!(stats.target_guilds, 1);
        assert_eq!(stats.planned, 0);
        assert_eq!(stats.skipped_channel_unresolved, 1);
        assert!(results.is_empty());
    }

    #[test]
    fn resolve_target_task_types_with_none_returns_all_task_types() {
        let result = resolve_target_task_types(None);

        assert_eq!(result.len(), 7);
        assert!(result.contains(&ScheduledTaskType::Notification));
        assert!(result.contains(&ScheduledTaskType::AutoMatching));
    }

    #[test]
    fn resolve_target_task_types_with_specific_returns_only_requested_task_type() {
        let result = resolve_target_task_types(Some(ScheduledTaskType::Dismissal));

        assert_eq!(result, vec![ScheduledTaskType::Dismissal]);
    }
}
