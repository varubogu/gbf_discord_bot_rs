//! 自動マッチングタスク実行サービス
//!
//! 10秒間隔で実行され、同じクエスト・時間・属性を希望するユーザーをマッチングし、
//! マッチング通知を送信後、マルチ募集を作成する

use crate::events::converters::to_create_message;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentRepository, SeaOrmQuestMatchingRepository,
    SeaOrmQuestMatchingUserRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::recruitment::recruitment_creation_service::{
    MatchingRecruitmentParams, RecruitmentCreationService,
};
use crate::types::discord::{EmbedContent, MessageContent};
use crate::types::{AppError, Result};
use chrono::{Datelike, Duration, TimeZone, Utc};
use poise::serenity_prelude::{ChannelId, Http};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// マッチング実行結果
#[derive(Debug, Clone, PartialEq)]
pub enum AutoMatchingResult {
    /// 実行成功
    Success {
        /// マッチしたグループ数
        matched_groups: usize,
        /// 次回タスクID
        next_task_id: i32,
    },
    /// マッチング対象なし
    NoMatches { next_task_id: i32 },
}

/// 自動マッチングタスク実行サービス
pub struct AutoMatchingTaskExecutor {
    task_repo: Arc<SeaOrmScheduledTaskRepository>,
    recruitment_creation_service: Arc<RecruitmentCreationService>,
}

impl AutoMatchingTaskExecutor {
    pub fn new(
        task_repo: Arc<SeaOrmScheduledTaskRepository>,
        recruitment_creation_service: Arc<RecruitmentCreationService>,
    ) -> Self {
        Self {
            task_repo,
            recruitment_creation_service,
        }
    }

    /// 自動マッチングタスクを実行する
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `db_conn` - データベース接続
    /// * `http` - Discord HTTP クライアント
    /// * `task_id` - 実行対象のタスクID
    ///
    /// # 戻り値
    /// * `Ok(AutoMatchingResult)` - 実行結果
    pub async fn execute(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        http: &Arc<Http>,
        task_id: i32,
    ) -> Result<AutoMatchingResult> {
        info!(task_id, "自動マッチングタスク実行開始");

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

        // マッチング処理を実行
        let matching_service = PeriodicMatchingService::new();
        let matchings = matching_service.process_matching(txn).await?;

        let matched_groups = matchings.len();

        // マッチング通知を送信し、マルチ募集を作成
        if !matchings.is_empty() {
            self.send_match_notifications_and_create_recruitments(txn, db_conn, http, &matchings)
                .await?;
        }

        // タスクを実行済みにマーク
        self.task_repo.mark_as_executed(txn, task_id).await?;

        // 次回タスクを作成（10秒後）
        let next_task_id = self.create_next_scheduled_task(txn).await?;

        if matched_groups > 0 {
            info!(
                task_id,
                matched_groups, next_task_id, "自動マッチングタスク実行完了"
            );
            Ok(AutoMatchingResult::Success {
                matched_groups,
                next_task_id,
            })
        } else {
            debug!(task_id, next_task_id, "マッチング対象なし");
            Ok(AutoMatchingResult::NoMatches { next_task_id })
        }
    }

    /// マッチング通知を送信し、マルチ募集を作成
    async fn send_match_notifications_and_create_recruitments(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        http: &Arc<Http>,
        matchings: &[crate::models::entities::worker::quest_matchings::Model],
    ) -> Result<()> {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let matching_user_repo = SeaOrmQuestMatchingUserRepository::new();
        let matching_repo = SeaOrmQuestMatchingRepository::new();
        let quest_repo = SeaOrmQuestRepository::new();

        // ギルドごとにグルーピング
        let mut guild_matchings: HashMap<i64, Vec<_>> = HashMap::new();
        for matching in matchings {
            guild_matchings
                .entry(matching.guild_id)
                .or_default()
                .push(matching);
        }

        for (guild_id, guild_matches) in guild_matchings {
            // 自動募集設定を取得
            let auto_recruitment = match auto_recruitment_repo
                .find_by_guild_id(txn, guild_id)
                .await?
            {
                Some(ar) => ar,
                None => {
                    warn!(guild_id, "自動募集設定が見つかりません");
                    continue;
                }
            };

            let matching_channel_id = match auto_recruitment.matching_channel_id {
                Some(id) => id as u64,
                None => {
                    warn!(guild_id, "マッチングチャンネルが設定されていません");
                    continue;
                }
            };

            for matching in guild_matches {
                // クエスト情報を取得
                let quest = match quest_repo.get_by_target_id(txn, matching.quest_id).await? {
                    Some(q) => q,
                    None => {
                        warn!(
                            guild_id,
                            quest_id = matching.quest_id,
                            "クエストが見つかりません"
                        );
                        continue;
                    }
                };

                // 参加ユーザーを取得
                let users = matching_user_repo
                    .find_active_by_matching(txn, guild_id, matching.id)
                    .await?;

                if users.is_empty() {
                    continue;
                }

                let user_ids: Vec<u64> = users.iter().map(|u| u.user_id as u64).collect();

                // 通知を送信
                if let Err(e) = self
                    .send_notification(
                        http,
                        matching_channel_id,
                        &quest.name,
                        matching.scheduled_month,
                        matching.scheduled_day,
                        matching.scheduled_hour,
                        &users
                            .iter()
                            .map(|u| (u.user_id as u64, u.battle_style_id))
                            .collect::<Vec<_>>(),
                    )
                    .await
                {
                    error!(
                        error = %e,
                        guild_id,
                        matching_id = %matching.id,
                        "マッチング通知の送信に失敗しました"
                    );
                    // 通知失敗しても募集作成は試みる
                }

                // 出発時刻を計算
                let quest_start_at = self.calculate_quest_start_at(
                    matching.scheduled_month,
                    matching.scheduled_day,
                    matching.scheduled_hour,
                );

                // 出発時刻が過去の場合はスキップ
                let now = Utc::now();
                if quest_start_at <= now {
                    info!(
                        guild_id,
                        matching_id = %matching.id,
                        quest_start_at = %quest_start_at,
                        now = %now,
                        "出発時刻が過去のためマルチ募集の作成をスキップしました"
                    );
                    continue;
                }

                // マルチ募集を作成
                let params = MatchingRecruitmentParams {
                    guild_id,
                    quest_id: matching.quest_id,
                    quest_start_at,
                    participant_user_ids: user_ids,
                };

                match self
                    .recruitment_creation_service
                    .create_recruitment_from_matching(txn, db_conn, http, &params)
                    .await
                {
                    Ok(recruitment_id) => {
                        info!(
                            guild_id,
                            matching_id = %matching.id,
                            recruitment_id,
                            "マルチ募集を作成しました"
                        );

                        // マッチングに募集IDを設定
                        if let Err(e) = matching_repo
                            .set_recruitment_id(txn, guild_id, matching.id, recruitment_id)
                            .await
                        {
                            error!(
                                error = %e,
                                guild_id,
                                matching_id = %matching.id,
                                recruitment_id,
                                "マッチングへの募集ID設定に失敗しました"
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            guild_id,
                            matching_id = %matching.id,
                            "マルチ募集の作成に失敗しました"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// スケジュール情報から出発時刻を計算
    fn calculate_quest_start_at(&self, month: i32, day: i32, hour: i32) -> chrono::DateTime<Utc> {
        // 現在の年を使用
        let now = Utc::now();
        let year = now.year();

        // hourが24以上の場合は翌日扱い（グラブルの5:00-28:00表記対応）
        let (actual_day, actual_hour) = if hour >= 24 {
            (day + 1, hour - 24)
        } else {
            (day, hour)
        };

        // 日本時間で構築してUTCに変換
        let jst = chrono_tz::Asia::Tokyo;
        let local_datetime = jst
            .with_ymd_and_hms(
                year,
                month as u32,
                actual_day as u32,
                actual_hour as u32,
                0,
                0,
            )
            .single()
            .unwrap_or_else(|| {
                // 年をまたぐ場合は翌年を試す
                jst.with_ymd_and_hms(
                    year + 1,
                    month as u32,
                    actual_day as u32,
                    actual_hour as u32,
                    0,
                    0,
                )
                .single()
                .expect("日時の構築に失敗しました")
            });

        local_datetime.with_timezone(&Utc)
    }

    /// 個別のマッチング通知を送信
    async fn send_notification(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
        users: &[(u64, Option<i32>)],
    ) -> Result<()> {
        let channel = ChannelId::new(channel_id);

        // 参加者メンション
        let participant_mentions: Vec<String> = users
            .iter()
            .map(|(user_id, _)| format!("<@{}>", user_id))
            .collect();

        // 属性情報を構築（6属性クエストの場合）
        let element_info = self.build_element_info(users);

        // Embed作成
        let embed_content = EmbedContent::new()
            .with_title("🎮 マッチング成立！")
            .with_description(&format!(
                "**クエスト**: {}\n**日時**: {}月{}日 {}:00\n\n**参加者**:\n{}{}\n\n募集を作成しています...",
                quest_name,
                month,
                day,
                hour,
                participant_mentions.join("\n"),
                element_info,
            ))
            .with_color(0x00ff00);

        let message_content = MessageContent::new()
            .with_text(&participant_mentions.join(" "))
            .with_embed(embed_content);

        channel
            .send_message(http, to_create_message(&message_content))
            .await
            .map_err(|e| AppError::Business {
                message: format!("マッチング通知の送信に失敗しました: {}", e),
            })?;

        info!(
            channel_id,
            quest_name,
            month,
            day,
            hour,
            user_count = users.len(),
            "マッチング通知を送信しました"
        );

        Ok(())
    }

    /// 属性情報を文字列化
    fn build_element_info(&self, users: &[(u64, Option<i32>)]) -> String {
        // 属性IDが設定されているかチェック
        let has_elements = users
            .iter()
            .any(|(_, style)| style.is_some() && *style != Some(0));

        if !has_elements {
            return String::new();
        }

        let element_names = [
            (1, "火"),
            (2, "水"),
            (3, "土"),
            (4, "風"),
            (5, "光"),
            (6, "闇"),
        ];
        let element_map: HashMap<i32, &str> = element_names.into_iter().collect();

        let elements: Vec<String> = users
            .iter()
            .filter_map(|(user_id, style)| {
                style.and_then(|s| {
                    if s > 0 {
                        element_map
                            .get(&s)
                            .map(|name| format!("<@{}>: {}", user_id, name))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if elements.is_empty() {
            String::new()
        } else {
            format!("\n\n**担当属性**:\n{}", elements.join("\n"))
        }
    }

    /// 次回実行タスクを作成（10秒後）
    async fn create_next_scheduled_task(&self, txn: &DatabaseTransaction) -> Result<i32> {
        let next_execution = Utc::now() + Duration::seconds(10);

        debug!(
            next_execution = %next_execution,
            "次回自動マッチングタスクを作成します"
        );

        let task = self
            .task_repo
            .create(
                txn,
                next_execution,
                ScheduledTaskType::AutoMatching as i32,
                None, // guild_id: 全ギルド対象
                None, // channel_id
            )
            .await?;

        debug!(
            task_id = task.id,
            next_execution = %next_execution,
            "次回自動マッチングタスクを作成しました"
        );

        Ok(task.id)
    }
}

#[cfg(test)]
mod tests {
    // TODO: モックを使ったテスト実装
}
