//! 自動募集日付ローテーションタスク実行サービス
//!
//! 日時チャンネルを「今日起点の連続日付」に補正し、
//! チャンネルを日付昇順に並び替える。

use crate::gateway::DiscordGateway;
use crate::models::entities::guild_master::{auto_recruitment_channels, auto_recruitments};
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, ChannelCreateParams, ChannelEditParams, DiscordChannelId, DiscordGuildId,
    DiscordMessageId, MessageContent, SelectMenuContent, SelectMenuOptionContent,
};
use crate::types::{AppError, Result};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use rust_i18n::t;
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// ローテーション実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum AutoRecruitmentRotationResult {
    /// 実行成功
    Success {
        rotated_channels: usize,
        next_task_id: i32,
    },
    /// 対象ギルドなし
    NoGuilds,
}

/// 起動時補正の実行結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupRepairResult {
    /// 補正対象ギルド数
    pub total_guilds: usize,
    /// 正常終了ギルド数
    pub repaired_guilds: usize,
    /// 起動時に作成された日時チャンネル数
    pub created_channels: usize,
    /// 日付更新が行われた日時チャンネル数
    pub rotated_channels: usize,
    /// 補正失敗ギルド数
    pub failed_guilds: usize,
}

/// 自動募集日付ローテーションタスク実行サービス
pub struct AutoRecruitmentRotationTaskExecutor<C, ST, AR>
where
    C: AutoRecruitmentChannelRepository,
    ST: ScheduledTaskRepository,
    AR: AutoRecruitmentRepository,
{
    task_repo: Arc<ST>,
    channel_repo: Arc<C>,
    auto_recruitment_repo: AR,
}

impl<C, ST, AR> AutoRecruitmentRotationTaskExecutor<C, ST, AR>
where
    C: AutoRecruitmentChannelRepository,
    ST: ScheduledTaskRepository,
    AR: AutoRecruitmentRepository,
{
    pub fn new(task_repo: Arc<ST>, channel_repo: Arc<C>, auto_recruitment_repo: AR) -> Self {
        Self {
            task_repo,
            channel_repo,
            auto_recruitment_repo,
        }
    }

    /// Bot起動時に日時チャンネルを補正する
    ///
    /// - `days_range` 未満の日時チャンネルを自動作成
    /// - 日時チャンネルを今日起点の連続日付へ再割当
    /// - チャンネル順序を再調整
    ///
    /// 一部ギルドで失敗しても処理を継続し、結果を返す。
    pub async fn repair_on_startup<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
    ) -> Result<StartupRepairResult> {
        let today = current_jst_date();
        info!(today = %today, "起動時の自動募集日時チャンネル補正を開始します");

        let auto_recruitments = self.auto_recruitment_repo.find_all(txn).await?;
        if auto_recruitments.is_empty() {
            info!("補正対象の自動募集設定がありません");
            return Ok(StartupRepairResult {
                total_guilds: 0,
                repaired_guilds: 0,
                created_channels: 0,
                rotated_channels: 0,
                failed_guilds: 0,
            });
        }

        let mut repaired_guilds = 0usize;
        let mut failed_guilds = 0usize;
        let mut created_channels = 0usize;
        let mut rotated_channels = 0usize;

        for auto_recruitment in &auto_recruitments {
            match self
                .repair_single_guild_on_startup(txn, gateway, auto_recruitment, today)
                .await
            {
                Ok((created, rotated)) => {
                    repaired_guilds += 1;
                    created_channels += created;
                    rotated_channels += rotated;
                }
                Err(e) => {
                    failed_guilds += 1;
                    error!(
                        guild_id = auto_recruitment.guild_id,
                        error = %e,
                        "起動時補正に失敗しました（処理は継続）"
                    );
                }
            }
        }

        info!(
            total_guilds = auto_recruitments.len(),
            repaired_guilds,
            failed_guilds,
            created_channels,
            rotated_channels,
            "起動時の自動募集日時チャンネル補正が完了しました"
        );

        Ok(StartupRepairResult {
            total_guilds: auto_recruitments.len(),
            repaired_guilds,
            created_channels,
            rotated_channels,
            failed_guilds,
        })
    }

    /// ローテーションタスクを実行する
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `gateway` - Discord Gateway
    /// * `task_id` - 実行対象のタスクID
    ///
    /// # 戻り値
    /// * `Ok(AutoRecruitmentRotationResult)` - 実行結果
    pub async fn execute<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        task_id: i32,
    ) -> Result<AutoRecruitmentRotationResult> {
        info!(task_id, "自動募集日付ローテーションタスク実行開始");

        // タスクが削除されていないか、既に実行済みでないかを確認
        let _task = match self.task_repo.find_by_id(txn, task_id).await? {
            Some(task) if task.execution_status.is_pending() => task,
            Some(_) => {
                warn!(task_id, "タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("Task {task_id} is not pending"),
                });
            }
            None => {
                warn!(task_id, "タスクが見つかりません");
                return Err(AppError::Business {
                    message: format!("Task {task_id} not found"),
                });
            }
        };

        // 全ギルドの自動募集チャンネルを取得してローテーション
        let channels = self.channel_repo.find_all(txn).await?;

        if channels.is_empty() {
            info!(task_id, "ローテーション対象のチャンネルがありません");
            self.task_repo.mark_as_succeeded(txn, task_id).await?;
            let next_task_id = self.create_next_scheduled_task(txn).await?;
            return Ok(AutoRecruitmentRotationResult::Success {
                rotated_channels: 0,
                next_task_id,
            });
        }

        let today = current_jst_date();
        let mut rotated_count = 0usize;
        let mut guild_channels: HashMap<i64, Vec<auto_recruitment_channels::Model>> =
            HashMap::new();

        for channel in channels {
            guild_channels
                .entry(channel.guild_id)
                .or_default()
                .push(channel);
        }

        for (guild_id, channels) in guild_channels {
            debug!(
                guild_id,
                channel_count = channels.len(),
                "ギルドの日付チャンネルを連続日付へ再割当します"
            );

            let rotated = self
                .reassign_channels_to_contiguous_dates(txn, gateway, today, &channels)
                .await?;
            rotated_count += rotated;

            if let Err(e) = self.reorder_channels_by_date(txn, gateway, guild_id).await {
                error!(guild_id, error = %e, "チャンネルの並び替えに失敗しました");
            }
        }

        self.task_repo.mark_as_succeeded(txn, task_id).await?;
        let next_task_id = self.create_next_scheduled_task(txn).await?;

        info!(
            task_id,
            rotated_channels = rotated_count,
            next_task_id,
            "自動募集日付ローテーションタスク実行完了"
        );

        Ok(AutoRecruitmentRotationResult::Success {
            rotated_channels: rotated_count,
            next_task_id,
        })
    }

    async fn repair_single_guild_on_startup<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        auto_recruitment: &auto_recruitments::Model,
        today: NaiveDate,
    ) -> Result<(usize, usize)> {
        let guild_id = auto_recruitment.guild_id;
        let required_days = auto_recruitment.days_range.max(0) as usize;
        let mut channels = self.channel_repo.find_by_guild_id(txn, guild_id).await?;

        let created_count = if channels.len() < required_days {
            let created = self
                .create_missing_channels_for_guild(
                    txn,
                    gateway,
                    auto_recruitment,
                    channels.len(),
                    required_days,
                    today,
                )
                .await;
            channels = self.channel_repo.find_by_guild_id(txn, guild_id).await?;
            created
        } else {
            0
        };

        let rotated_count = self
            .reassign_channels_to_contiguous_dates(txn, gateway, today, &channels)
            .await?;

        if let Err(e) = self.reorder_channels_by_date(txn, gateway, guild_id).await {
            warn!(guild_id, error = %e, "起動時補正の並び替えに失敗しました");
        }

        Ok((created_count, rotated_count))
    }

    async fn create_missing_channels_for_guild<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        auto_recruitment: &auto_recruitments::Model,
        existing_count: usize,
        required_days: usize,
        today: NaiveDate,
    ) -> usize {
        let missing_count = calculate_missing_channel_count(required_days, existing_count);
        if missing_count == 0 {
            return 0;
        }

        let guild_id = auto_recruitment.guild_id;
        let category_id = auto_recruitment.category_id as u64;
        let discord_guild_id = DiscordGuildId::new(guild_id as u64);
        let mut created_count = 0usize;

        for offset in 0..missing_count {
            let index = existing_count + offset;
            let target_date = today + Duration::days(index as i64);
            let channel_name = format!("{}月{}日", target_date.month(), target_date.day());
            let channel_position = (index + 1) as u16;

            let create_params = ChannelCreateParams::text(&channel_name)
                .with_parent(DiscordChannelId::new(category_id))
                .with_position(channel_position);

            let created_channel = match gateway
                .create_channel(discord_guild_id, create_params)
                .await
            {
                Ok(channel_id) => channel_id,
                Err(e) => {
                    warn!(
                        guild_id,
                        category_id,
                        error = %e,
                        "起動時補正: 日時チャンネルの作成に失敗しました"
                    );
                    continue;
                }
            };

            let message_id = match self
                .send_time_selection_message(gateway, created_channel)
                .await
            {
                Ok(msg_id) => Some(msg_id.get() as i64),
                Err(e) => {
                    warn!(
                        guild_id,
                        channel_id = created_channel.get(),
                        error = %e,
                        "起動時補正: 時間選択メッセージ送信に失敗しました"
                    );
                    None
                }
            };

            if let Err(e) = self
                .channel_repo
                .create(
                    txn,
                    guild_id,
                    created_channel.get() as i64,
                    target_date.month() as i32,
                    target_date.day() as i32,
                    index as i32,
                    true,
                    message_id,
                )
                .await
            {
                error!(
                    guild_id,
                    channel_id = created_channel.get(),
                    error = %e,
                    "起動時補正: 作成した日時チャンネルのDB登録に失敗しました"
                );
                continue;
            }

            created_count += 1;
            info!(
                guild_id,
                channel_id = created_channel.get(),
                "起動時補正: 日時チャンネルを作成しました"
            );
        }

        created_count
    }

    async fn send_time_selection_message<G: DiscordGateway>(
        &self,
        gateway: &G,
        channel_id: DiscordChannelId,
    ) -> Result<DiscordMessageId> {
        // ゲーム内日付: 当日5:00〜翌日4:00（24時間）
        // 降順で表示: 翌4:00, 翌3:00, ... 翌0:00, 23:00, ... 5:00
        let mut options: Vec<SelectMenuOptionContent> = Vec::with_capacity(24);

        for hour in (0..=4).rev() {
            let label = format!("翌{hour}:00");
            let value = (24 + hour).to_string();
            options.push(SelectMenuOptionContent::new(label, value));
        }

        for hour in (5..=23).rev() {
            let label = format!("{hour}:00");
            let value = hour.to_string();
            options.push(SelectMenuOptionContent::new(label, value));
        }

        let placeholder = t!(
            MessageTextId::AutoRecruitmentTimeSelectPlaceholder.as_str(),
            locale = "ja"
        )
        .to_string();

        let custom_id = format!("auto_time_select:{}", channel_id.get());
        let select_menu = SelectMenuContent::string_select(&custom_id, options)
            .with_placeholder(&placeholder)
            .with_min_values(0)
            .with_max_values(24);
        let action_row = ActionRowContent::select_menu(select_menu);

        let message_content = MessageContent::new()
            .with_text("**参加可能な時間帯を選択してください**\n複数選択可能です。選択を変更すると自動的に更新されます。")
            .with_component(action_row);

        gateway
            .send_message(channel_id, message_content)
            .await
            .map_err(|e| AppError::Business {
                message: format!("時間選択メッセージの送信に失敗しました: {e}"),
            })
    }

    async fn reassign_channels_to_contiguous_dates<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        today: NaiveDate,
        channels: &[auto_recruitment_channels::Model],
    ) -> Result<usize> {
        if channels.is_empty() {
            return Ok(0);
        }

        let mut channels = channels.to_vec();
        sort_channels_by_logical_date(today, &mut channels);

        let mut rotated_count = 0usize;
        for (idx, channel) in channels.iter().enumerate() {
            let target_date = today + Duration::days(idx as i64);
            let target_month = target_date.month() as i32;
            let target_day = target_date.day() as i32;

            if channel.month == target_month && channel.day == target_day {
                continue;
            }

            self.channel_repo
                .update_date(txn, channel.id, target_month, target_day)
                .await?;

            let target_name = format!("{target_month}月{target_day}日");
            if let Err(e) = self
                .update_discord_channel_name(gateway, channel.channel_id as u64, &target_name)
                .await
            {
                warn!(
                    channel_id = channel.channel_id,
                    error = %e,
                    "日時チャンネル名の更新に失敗しました"
                );
            }

            rotated_count += 1;
        }

        Ok(rotated_count)
    }

    /// Discordチャンネル名を更新（Gateway経由）
    async fn update_discord_channel_name<G: DiscordGateway>(
        &self,
        gateway: &G,
        channel_id: u64,
        new_name: &str,
    ) -> Result<()> {
        let channel_id = DiscordChannelId::new(channel_id);
        let params = ChannelEditParams::new().with_name(new_name);

        gateway
            .edit_channel(channel_id, params)
            .await
            .map_err(|e| AppError::Business {
                message: format!("チャンネル名の更新に失敗しました: {e}"),
            })?;

        Ok(())
    }

    /// チャンネルを日付昇順に並び替え（Gateway経由）
    ///
    /// 順序: マッチング(0) → 日付昇順(1〜n) → クエスト(n+1)
    async fn reorder_channels_by_date<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        gateway: &G,
        guild_id: i64,
    ) -> Result<()> {
        let mut channels = self.channel_repo.find_by_guild_id(txn, guild_id).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let auto_recruitment = self
            .auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?;

        if let Some(ref ar) = auto_recruitment
            && let Some(matching_ch_id) = ar.matching_channel_id
        {
            let matching_channel_id = DiscordChannelId::new(matching_ch_id as u64);
            let params = ChannelEditParams::new().with_position(0);
            if let Err(e) = gateway.edit_channel(matching_channel_id, params).await {
                warn!(
                    channel_id = matching_ch_id,
                    error = %e,
                    "マッチングチャンネル位置の更新に失敗しました"
                );
            }
        }

        let today = current_jst_date();
        sort_channels_by_logical_date(today, &mut channels);

        for (idx, channel) in channels.iter().enumerate() {
            let discord_channel_id = DiscordChannelId::new(channel.channel_id as u64);
            let position = (idx + 1) as u16;
            let params = ChannelEditParams::new().with_position(position);

            if let Err(e) = gateway.edit_channel(discord_channel_id, params).await {
                warn!(
                    channel_id = channel.channel_id,
                    position,
                    error = %e,
                    "日時チャンネル位置の更新に失敗しました"
                );
            }
        }

        if let Some(ref ar) = auto_recruitment
            && let Some(quest_ch_id) = ar.quest_channel_id
        {
            let quest_channel_id = DiscordChannelId::new(quest_ch_id as u64);
            let quest_position = (channels.len() + 1) as u16;
            let params = ChannelEditParams::new().with_position(quest_position);
            if let Err(e) = gateway.edit_channel(quest_channel_id, params).await {
                warn!(
                    channel_id = quest_ch_id,
                    position = quest_position,
                    error = %e,
                    "クエストチャンネル位置の更新に失敗しました"
                );
            }
        }

        Ok(())
    }

    /// 次回実行タスクを作成（翌日0時）
    async fn create_next_scheduled_task(&self, txn: &DatabaseTransaction) -> Result<i32> {
        let next_execution = next_rotation_execution_datetime(Utc::now());

        debug!(
            next_execution = %next_execution,
            "次回ローテーションタスクを作成します"
        );

        let task = self
            .task_repo
            .create(
                txn,
                next_execution,
                ScheduledTaskType::AutoRecruitmentRotation as i32,
                None,
                None,
            )
            .await?;

        info!(
            task_id = task.id,
            next_execution = %next_execution,
            "次回ローテーションタスクを作成しました"
        );

        Ok(task.id)
    }
}

fn current_jst_date() -> NaiveDate {
    let now_jst = Utc::now() + Duration::hours(9);
    now_jst.date_naive()
}

fn next_rotation_execution_datetime(now_utc: DateTime<Utc>) -> DateTime<Utc> {
    let now_jst = now_utc + Duration::hours(9);
    let tomorrow_jst = (now_jst + Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_else(|| now_jst.naive_utc());
    let next_execution_utc = tomorrow_jst - Duration::hours(9);
    DateTime::<Utc>::from_naive_utc_and_offset(next_execution_utc, Utc)
}

fn calculate_missing_channel_count(required_days: usize, existing_count: usize) -> usize {
    required_days.saturating_sub(existing_count)
}

fn resolve_channel_logical_date(
    today: NaiveDate,
    channel: &auto_recruitment_channels::Model,
) -> Option<NaiveDate> {
    let base = NaiveDate::from_ymd_opt(today.year(), channel.month as u32, channel.day as u32)?;
    if base < today && (channel.month as u32) < today.month() {
        NaiveDate::from_ymd_opt(today.year() + 1, channel.month as u32, channel.day as u32)
            .or(Some(base))
    } else {
        Some(base)
    }
}

fn sort_channels_by_logical_date(
    today: NaiveDate,
    channels: &mut [auto_recruitment_channels::Model],
) {
    channels.sort_by(|a, b| {
        let date_a = resolve_channel_logical_date(today, a);
        let date_b = resolve_channel_logical_date(today, b);

        match (date_a, date_b) {
            (Some(da), Some(db)) => da
                .cmp(&db)
                .then_with(|| a.sort_order.cmp(&b.sort_order))
                .then_with(|| a.id.cmp(&b.id)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .sort_order
                .cmp(&b.sort_order)
                .then_with(|| a.id.cmp(&b.id)),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(
        id: i32,
        guild_id: i64,
        month: i32,
        day: i32,
        sort_order: i32,
    ) -> auto_recruitment_channels::Model {
        let now = Utc::now();
        auto_recruitment_channels::Model {
            id,
            guild_id,
            channel_id: 1000 + id as i64,
            month,
            day,
            sort_order,
            is_bot_created: true,
            message_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_sort_channels_by_logical_date_year_boundary() {
        let today = NaiveDate::from_ymd_opt(2026, 12, 30).expect("valid date");
        let mut channels = vec![
            channel(1, 1, 1, 1, 1),
            channel(2, 1, 12, 29, 0),
            channel(3, 1, 12, 31, 2),
        ];

        sort_channels_by_logical_date(today, &mut channels);

        // 12/29(過去) -> 12/31 -> 1/1(翌年) の順になる
        assert_eq!(channels[0].id, 2);
        assert_eq!(channels[1].id, 3);
        assert_eq!(channels[2].id, 1);
    }

    #[test]
    fn test_reassign_target_dates_cover_seven_days_after_long_stop() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 19).expect("valid date");
        let mut channels = vec![
            channel(1, 1, 2, 1, 0),
            channel(2, 1, 2, 2, 1),
            channel(3, 1, 2, 3, 2),
            channel(4, 1, 2, 4, 3),
            channel(5, 1, 2, 5, 4),
            channel(6, 1, 2, 6, 5),
            channel(7, 1, 2, 7, 6),
        ];
        sort_channels_by_logical_date(today, &mut channels);

        let targets: Vec<_> = channels
            .iter()
            .enumerate()
            .map(|(idx, _)| today + Duration::days(idx as i64))
            .collect();

        assert_eq!(targets.len(), 7);
        assert_eq!(targets[0], today);
        assert_eq!(targets[6], today + Duration::days(6));
    }

    #[test]
    fn test_excess_channels_do_not_duplicate_target_dates() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 19).expect("valid date");
        let mut channels = vec![
            channel(1, 1, 2, 19, 0),
            channel(2, 1, 2, 19, 1),
            channel(3, 1, 2, 19, 2),
            channel(4, 1, 2, 20, 3),
            channel(5, 1, 2, 21, 4),
            channel(6, 1, 2, 22, 5),
            channel(7, 1, 2, 23, 6),
            channel(8, 1, 2, 24, 7),
        ];
        sort_channels_by_logical_date(today, &mut channels);

        let mut targets: Vec<_> = channels
            .iter()
            .enumerate()
            .map(|(idx, _)| today + Duration::days(idx as i64))
            .collect();
        targets.sort();
        targets.dedup();

        assert_eq!(targets.len(), channels.len());
    }

    #[test]
    fn test_calculate_missing_channel_count() {
        assert_eq!(calculate_missing_channel_count(7, 5), 2);
        assert_eq!(calculate_missing_channel_count(7, 7), 0);
        assert_eq!(calculate_missing_channel_count(7, 9), 0);
    }

    #[test]
    fn test_next_rotation_execution_datetime_is_next_jst_midnight() {
        // 2026-02-19 10:00 JST = 2026-02-19 01:00 UTC
        let now_utc = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2026, 2, 19)
                .expect("valid date")
                .and_hms_opt(1, 0, 0)
                .expect("valid time"),
            Utc,
        );

        let next = next_rotation_execution_datetime(now_utc);
        // 翌日0:00 JST = 当日15:00 UTC
        let expected = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2026, 2, 19)
                .expect("valid date")
                .and_hms_opt(15, 0, 0)
                .expect("valid time"),
            Utc,
        );

        assert_eq!(next, expected);
    }
}
