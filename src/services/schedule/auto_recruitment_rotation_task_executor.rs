//! 自動募集日付ローテーションタスク実行サービス
//!
//! 毎日0時に実行され、過去日のチャンネル名を新しい日付に更新し、
//! チャンネルを日付昇順に並び替える

use crate::gateway::DiscordGateway;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentRepository,
};
use crate::repository::database::auto_recruitment::SeaOrmAutoRecruitmentRepository;
use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;
use crate::repository::schedule::ScheduledTaskRepository;
use crate::types::discord::{ChannelEditParams, DiscordChannelId};
use crate::types::{AppError, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use sea_orm::DatabaseTransaction;
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

/// 自動募集日付ローテーションタスク実行サービス
pub struct AutoRecruitmentRotationTaskExecutor<C>
where
    C: AutoRecruitmentChannelRepository,
{
    task_repo: Arc<SeaOrmScheduledTaskRepository>,
    channel_repo: Arc<C>,
}

impl<C> AutoRecruitmentRotationTaskExecutor<C>
where
    C: AutoRecruitmentChannelRepository,
{
    pub fn new(task_repo: Arc<SeaOrmScheduledTaskRepository>, channel_repo: Arc<C>) -> Self {
        Self {
            task_repo,
            channel_repo,
        }
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
            Some(task) if !task.is_executed => task,
            Some(_) => {
                warn!(task_id, "タスクは既に実行済みです");
                return Err(AppError::Business {
                    message: format!("Task {task_id} is already executed"),
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
            // タスクを実行済みにマーク
            self.task_repo.mark_as_executed(txn, task_id).await?;
            // 次回タスクを作成
            let next_task_id = self.create_next_scheduled_task(txn).await?;
            return Ok(AutoRecruitmentRotationResult::Success {
                rotated_channels: 0,
                next_task_id,
            });
        }

        // 今日の日付を取得（JST）
        let now_utc = Utc::now();
        // UTC+9 = JSTとして計算
        let now_jst = now_utc + Duration::hours(9);
        let today = now_jst.date_naive();

        let mut rotated_count = 0;

        // ギルドIDでグループ化
        let mut guild_channels: std::collections::HashMap<
            i64,
            Vec<crate::models::entities::guild_master::auto_recruitment_channels::Model>,
        > = std::collections::HashMap::new();

        for channel in channels {
            guild_channels
                .entry(channel.guild_id)
                .or_default()
                .push(channel);
        }

        // 各ギルドのチャンネルをローテーション
        for (guild_id, mut channels) in guild_channels {
            debug!(
                guild_id,
                channel_count = channels.len(),
                "ギルドのチャンネルをローテーションします"
            );

            // 日付でソート（古い順）
            channels.sort_by_key(|c| (c.month, c.day));

            // 過去日のチャンネルを検出して新しい日付に更新
            for channel in &channels {
                let channel_date = match NaiveDate::from_ymd_opt(
                    today.year(),
                    channel.month as u32,
                    channel.day as u32,
                ) {
                    Some(d) => d,
                    None => {
                        // 無効な日付（2/30など）の場合はスキップ
                        warn!(
                            channel_id = channel.id,
                            month = channel.month,
                            day = channel.day,
                            "無効な日付のチャンネルをスキップします"
                        );
                        continue;
                    }
                };

                if channel_date < today {
                    // 過去日のチャンネルを最も未来の日付に更新
                    let new_date = self.calculate_new_date(&channels, today)?;

                    debug!(
                        channel_id = channel.id,
                        old_date = %channel_date,
                        new_date = %new_date,
                        "チャンネルの日付を更新します"
                    );

                    // DBを更新
                    self.channel_repo
                        .update_date(
                            txn,
                            channel.id,
                            new_date.month() as i32,
                            new_date.day() as i32,
                        )
                        .await?;

                    // Discordチャンネル名を更新
                    let new_channel_name = format!("{}月{}日", new_date.month(), new_date.day());
                    if let Err(e) = self
                        .update_discord_channel_name(
                            gateway,
                            channel.channel_id as u64,
                            &new_channel_name,
                        )
                        .await
                    {
                        error!(
                            channel_id = channel.channel_id,
                            error = %e,
                            "Discordチャンネル名の更新に失敗しました"
                        );
                        // 失敗しても続行
                    }

                    rotated_count += 1;
                }
            }

            // チャンネルを日付昇順に並び替え
            if let Err(e) = self.reorder_channels_by_date(txn, gateway, guild_id).await {
                error!(
                    guild_id,
                    error = %e,
                    "チャンネルの並び替えに失敗しました"
                );
                // 失敗しても続行
            }
        }

        // タスクを実行済みにマーク
        self.task_repo.mark_as_executed(txn, task_id).await?;

        // 次回タスクを作成
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

    /// 新しい日付を計算
    ///
    /// 既存のチャンネル日付の中で最も未来の日付の翌日を返す
    fn calculate_new_date(
        &self,
        channels: &[crate::models::entities::guild_master::auto_recruitment_channels::Model],
        today: NaiveDate,
    ) -> Result<NaiveDate> {
        let mut max_date = today;

        for channel in channels {
            if let Some(date) =
                NaiveDate::from_ymd_opt(today.year(), channel.month as u32, channel.day as u32)
            {
                // 年跨ぎを考慮：チャンネル日付が今日より過去で、月が今日より大きい場合は来年
                let channel_date = if date < today && channel.month as u32 > today.month() {
                    NaiveDate::from_ymd_opt(
                        today.year() + 1,
                        channel.month as u32,
                        channel.day as u32,
                    )
                    .unwrap_or(date)
                } else {
                    date
                };

                if channel_date > max_date {
                    max_date = channel_date;
                }
            }
        }

        // 最大日付の翌日を返す
        Ok(max_date + Duration::days(1))
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
                message: format!("チャンネル名の更新に失敗しました: {}", e),
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

        // 自動募集設定を取得（マッチング/クエストチャンネルID）
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?;

        // マッチングチャンネルをposition 0に設定
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

        // 今日の日付を基準に日付でソート
        let now_utc = Utc::now();
        let now_jst = now_utc + Duration::hours(9);
        let today = now_jst.date_naive();

        channels.sort_by(|a, b| {
            let date_a = NaiveDate::from_ymd_opt(today.year(), a.month as u32, a.day as u32)
                .unwrap_or(today);
            let date_b = NaiveDate::from_ymd_opt(today.year(), b.month as u32, b.day as u32)
                .unwrap_or(today);

            // 年跨ぎを考慮
            let date_a = if date_a < today && a.month as u32 > today.month() {
                NaiveDate::from_ymd_opt(today.year() + 1, a.month as u32, a.day as u32)
                    .unwrap_or(date_a)
            } else {
                date_a
            };
            let date_b = if date_b < today && b.month as u32 > today.month() {
                NaiveDate::from_ymd_opt(today.year() + 1, b.month as u32, b.day as u32)
                    .unwrap_or(date_b)
            } else {
                date_b
            };

            date_a.cmp(&date_b)
        });

        // 日付チャンネルの位置を更新（position 1から）
        for (i, channel) in channels.iter().enumerate() {
            let discord_channel_id = DiscordChannelId::new(channel.channel_id as u64);
            let position = (i + 1) as u16; // position 1から開始
            let params = ChannelEditParams::new().with_position(position);

            if let Err(e) = gateway.edit_channel(discord_channel_id, params).await {
                warn!(
                    channel_id = channel.channel_id,
                    position,
                    error = %e,
                    "チャンネル位置の更新に失敗しました"
                );
                // 失敗しても続行
            }
        }

        // クエストチャンネルを日付チャンネルの後に設定
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
        // 翌日の0時（JST）をUTCに変換
        let now_utc = Utc::now();
        let now_jst = now_utc + Duration::hours(9);
        let tomorrow_jst = (now_jst + Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        // JST 0時 = UTC 15時（前日）
        let next_execution_utc = tomorrow_jst - Duration::hours(9);
        let next_execution =
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(next_execution_utc, Utc);

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
                None, // guild_id: 全ギルド対象
                None, // channel_id
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

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
