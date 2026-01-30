use crate::events::converters::{to_create_action_row, to_create_embed};
use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::presenter::RecruitmentPresenter;
use crate::repository::GuildChannelRepository;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::guild_channel_repository::SeaOrmGuildChannelRepository;
use crate::repository::database::guild_environment_repository::SeaOrmGuildEnvironmentRepository;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::SeaOrmBattleRecruitmentScheduleDismissalRepository;
use crate::repository::quest_repository::QuestRepository;
use crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::new::create_message_content;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{DismissalManagementService, NotificationManagementService};
use crate::services::timezone_service::TimezoneService;
use crate::services::unified_datetime_parser::ParsedDismissalTime;
use crate::types::discord::EmbedContent;
use crate::types::Result;
use chrono::{TimeZone, Utc};
use poise::serenity_prelude::{CreateMessage, Http};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;
use tracing::{debug, info};

/// マッチングから募集を作成するためのパラメータ
pub struct MatchingRecruitmentParams {
    /// ギルドID
    pub guild_id: i64,
    /// クエストID
    pub quest_id: i32,
    /// 出発時刻（UTC）
    pub quest_start_at: chrono::DateTime<Utc>,
    /// 参加ユーザーID一覧（メンション用）
    pub participant_user_ids: Vec<u64>,
}

/// 募集作成Service
/// スケジュールから募集を作成する責務を持つ
pub struct RecruitmentCreationService;

impl Default for RecruitmentCreationService {
    fn default() -> Self {
        Self::new()
    }
}

impl RecruitmentCreationService {
    pub fn new() -> Self {
        Self
    }

    /// スケジュールから募集を作成
    /// 定期募集スケジュールに基づいて実際の募集を作成
    pub async fn create_recruitment_from_schedule(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        http: &Arc<Http>,
        calculated_time: &crate::services::schedule::CalculatedRecruitmentTime,
    ) -> Result<()> {
        debug!(
            schedule_id = calculated_time.schedule_id,
            quest_id = calculated_time.quest_id,
            "スケジュールから募集を作成します"
        );

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(txn, calculated_time.guild_id).await?;

        // 0. マルチ募集チャンネルを取得（channel_type = 2）
        let guild_channel_repo = SeaOrmGuildChannelRepository::new();
        let guild_channel = guild_channel_repo
            .get_by_guild_and_type_with_txn(txn, calculated_time.guild_id, 2)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "ギルドID {} にマルチ募集チャンネルが登録されていません",
                    calculated_time.guild_id
                ))
            })?;

        let recruitment_channel_id = guild_channel.channel_id;
        debug!(
            recruitment_channel_id = recruitment_channel_id,
            "マルチ募集チャンネルを取得しました"
        );

        // 1. Quest, BattleStyle, タイムゾーンを取得
        let quest_repo = SeaOrmQuestRepository::new();
        let battle_style_repo = SeaOrmBattleStyleRepository::new();
        let timezone_repo = Arc::new(SeaOrmGuildSettingsRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);

        let quest = quest_repo
            .get_by_target_id(db_conn, calculated_time.quest_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    calculated_time.quest_id
                ))
            })?;

        let battle_style = battle_style_repo
            .get_by_id(db_conn, calculated_time.battle_style_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "攻略方法ID {} が見つかりませんでした",
                    calculated_time.battle_style_id
                ))
            })?;

        let timezone = timezone_service
            .get_guild_timezone(db_conn, calculated_time.guild_id)
            .await?;

        // 2. ロールメンションを取得
        let role_service = RoleNotificationService::new();
        let role_mentions = role_service
            .get_role_mentions(txn, calculated_time.guild_id, quest.id)
            .await?;

        // 2.5. 定期募集スケジュールの解散時刻を取得
        let dismissal_repo = SeaOrmBattleRecruitmentScheduleDismissalRepository::new();
        let schedule_dismissals = dismissal_repo
            .find_by_schedule_id(txn, calculated_time.schedule_id)
            .await?;

        // 解散時刻をParsedDismissalTimeに変換
        let parsed_dismissal_times: Vec<ParsedDismissalTime> = schedule_dismissals
            .iter()
            .map(|sd| {
                if sd.input_type == 1 {
                    // 絶対時刻
                    let dismissal_time =
                        sd.dismissal_time
                            .ok_or_else(|| crate::types::AppError::Business {
                                message: "絶対時刻の解散時刻が設定されていません".to_string(),
                            })?;
                    // TimeTimeをNaiveTimeに変換
                    let naive_time = chrono::NaiveTime::from_hms_opt(
                        dismissal_time.hour() as u32,
                        dismissal_time.minute() as u32,
                        dismissal_time.second() as u32,
                    )
                    .ok_or_else(|| crate::types::AppError::Business {
                        message: "解散時刻の変換に失敗しました".to_string(),
                    })?;
                    // 日付を出発日に合わせる
                    let departure_date = calculated_time.quest_start_at.date_naive();
                    let dismissal_datetime_local = timezone
                        .from_local_datetime(&departure_date.and_time(naive_time))
                        .single()
                        .ok_or_else(|| crate::types::AppError::Business {
                            message: "解散時刻の日時変換に失敗しました".to_string(),
                        })?;
                    // 出発時刻より後になる場合は前日にする
                    let dismissal_datetime_utc = if dismissal_datetime_local
                        >= calculated_time.quest_start_at.with_timezone(&timezone)
                    {
                        (dismissal_datetime_local - chrono::Duration::days(1))
                            .with_timezone(&chrono::Utc)
                    } else {
                        dismissal_datetime_local.with_timezone(&chrono::Utc)
                    };
                    Ok(ParsedDismissalTime::Absolute {
                        input_value: sd.input_value.clone(),
                        datetime: dismissal_datetime_utc,
                    })
                } else {
                    // 相対時刻
                    Ok(ParsedDismissalTime::Relative {
                        input_value: sd.input_value.clone(),
                        days: sd.relative_days.unwrap_or(0),
                        hours: sd.relative_hours.unwrap_or(0),
                        minutes: sd.relative_minutes.unwrap_or(0),
                    })
                }
            })
            .collect::<Result<Vec<_>>>()?;

        // 3. メッセージ内容を作成
        let dismissal_times_option = if parsed_dismissal_times.is_empty() {
            None
        } else {
            Some(parsed_dismissal_times.as_slice())
        };

        let mut message_content = create_message_content(
            txn,
            &quest.name,
            &battle_style.display_name,
            &calculated_time.quest_start_at,
            timezone,
            Some(calculated_time.guild_id),
            dismissal_times_option,
        )
        .await?;

        // 備考がある場合は追加
        if let Some(note) = &calculated_time.note {
            message_content.push_str(&format!("\n備考: {note}"));
        }

        // ロールメンションを先頭に追加
        if !role_mentions.is_empty() {
            debug!(
                role_mentions = %role_mentions,
                "ロールメンションを募集メッセージの先頭に追加します"
            );
            message_content = format!("{role_mentions}\n{message_content}");
        }

        // 3.5. 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service
            .get_element_emojis(txn, http, calculated_time.guild_id)
            .await?;

        // 4. Embedを作成（Presenterを使用）
        let initial_participants_text = RecruitmentPresenter::create_initial_participants_text(
            &battle_style.display_name,
            &element_emojis,
        );
        let embed_content = EmbedContent::new()
            .with_title("参加者一覧")
            .with_description(&initial_participants_text)
            .with_color(0x0099ff);

        // 5. ボタンを作成（PresenterのドメインモデルをConverterで変換）
        let button_components =
            RecruitmentPresenter::create_recruitment_buttons(&battle_style.display_name, &element_emojis);
        let buttons: Vec<_> = button_components.iter().map(to_create_action_row).collect();

        // 6. Discordメッセージを投稿（マルチ募集チャンネルに投稿）
        let channel_id = poise::serenity_prelude::ChannelId::new(recruitment_channel_id as u64);
        let message = channel_id
            .send_message(
                http,
                CreateMessage::new()
                    .content(message_content)
                    .embed(to_create_embed(&embed_content))
                    .components(buttons),
            )
            .await?;

        let message_id = message.id.get();

        debug!(
            message_id = %message_id,
            "Discordメッセージを投稿しました"
        );

        // 7. battle_recruitmentsに保存
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        let recruitment = battle_recruitment_repo
            .create_with_txn(
                txn,
                crate::repository::CreateBattleRecruitmentParams {
                    guild_id: calculated_time.guild_id as u64,
                    channel_id: recruitment_channel_id as u64,
                    message_id,
                    quest_id: quest.id,
                    battle_style_id: calculated_time.battle_style_id,
                    quest_start_at: calculated_time.quest_start_at,
                },
            )
            .await?;

        info!(
            recruitment_id = recruitment.id,
            "募集をデータベースに登録しました"
        );

        // 8. 出発時刻の通知を登録（5分前とちょうどの時刻）
        debug!(
            quest_start_at = %calculated_time.quest_start_at,
            "募集の出発通知を登録します"
        );

        let notification_management_service = NotificationManagementService::new();
        notification_management_service
            .create_recruitment_departure_notification(
                txn,
                calculated_time.quest_start_at,
                calculated_time.guild_id,
                recruitment_channel_id,
                recruitment.id,
            )
            .await?;

        // 9. 解散時刻を登録（指定されている場合）
        if !parsed_dismissal_times.is_empty() {
            debug!(
                recruitment_id = recruitment.id,
                dismissal_count = parsed_dismissal_times.len(),
                "募集の解散時刻を登録します"
            );

            let dismissal_service = DismissalManagementService::new();
            dismissal_service
                .create_recruitment_dismissals(
                    txn,
                    recruitment.id,
                    parsed_dismissal_times.clone(),
                    calculated_time.quest_start_at,
                    calculated_time.guild_id,
                    recruitment_channel_id,
                )
                .await?;

            info!(
                recruitment_id = recruitment.id,
                dismissal_count = parsed_dismissal_times.len(),
                "募集の解散時刻を登録しました"
            );
        }

        Ok(())
    }

    /// マッチングから募集を作成
    /// マッチング成立時に自動的に募集を作成
    ///
    /// # 戻り値
    /// 作成された募集のID
    pub async fn create_recruitment_from_matching(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        http: &Arc<Http>,
        params: &MatchingRecruitmentParams,
    ) -> Result<i32> {
        debug!(
            guild_id = params.guild_id,
            quest_id = params.quest_id,
            "マッチングから募集を作成します"
        );

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(txn, params.guild_id).await?;

        // 0. マルチ募集チャンネルを取得（channel_type = 2）
        let guild_channel_repo = SeaOrmGuildChannelRepository::new();
        let guild_channel = guild_channel_repo
            .get_by_guild_and_type_with_txn(txn, params.guild_id, 2)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "ギルドID {} にマルチ募集チャンネルが登録されていません",
                    params.guild_id
                ))
            })?;

        let recruitment_channel_id = guild_channel.channel_id;
        debug!(
            recruitment_channel_id = recruitment_channel_id,
            "マルチ募集チャンネルを取得しました"
        );

        // 1. Quest, BattleStyle（デフォルト）, タイムゾーンを取得
        let quest_repo = SeaOrmQuestRepository::new();
        let battle_style_repo = SeaOrmBattleStyleRepository::new();
        let timezone_repo = Arc::new(SeaOrmGuildSettingsRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);

        let quest = quest_repo
            .get_by_target_id(db_conn, params.quest_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    params.quest_id
                ))
            })?;

        // クエストのデフォルト攻略方法を使用
        let battle_style = battle_style_repo
            .get_by_id(db_conn, quest.default_battle_style_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "攻略方法ID {} が見つかりませんでした",
                    quest.default_battle_style_id
                ))
            })?;

        let timezone = timezone_service
            .get_guild_timezone(db_conn, params.guild_id)
            .await?;

        // 2. ロールメンションを取得
        let role_service = RoleNotificationService::new();
        let role_mentions = role_service
            .get_role_mentions(txn, params.guild_id, quest.id)
            .await?;

        // 3. メッセージ内容を作成（マッチングでは解散時刻なし）
        let mut message_content = create_message_content(
            txn,
            &quest.name,
            &battle_style.display_name,
            &params.quest_start_at,
            timezone,
            Some(params.guild_id),
            None, // 解散時刻なし
        )
        .await?;

        // ロールメンションを先頭に追加
        if !role_mentions.is_empty() {
            debug!(
                role_mentions = %role_mentions,
                "ロールメンションを募集メッセージの先頭に追加します"
            );
            message_content = format!("{role_mentions}\n{message_content}");
        }

        // マッチングユーザーのメンションを追加
        if !params.participant_user_ids.is_empty() {
            let user_mentions: Vec<String> = params
                .participant_user_ids
                .iter()
                .map(|user_id| format!("<@{}>", user_id))
                .collect();
            message_content = format!("{}\n{message_content}", user_mentions.join(" "));
        }

        // 3.5. 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service
            .get_element_emojis(txn, http, params.guild_id)
            .await?;

        // 4. Embedを作成（Presenterを使用）
        let initial_participants_text = RecruitmentPresenter::create_initial_participants_text(
            &battle_style.display_name,
            &element_emojis,
        );
        let embed_content = EmbedContent::new()
            .with_title("参加者一覧")
            .with_description(&initial_participants_text)
            .with_color(0x0099ff);

        // 5. ボタンを作成（PresenterのドメインモデルをConverterで変換）
        let button_components =
            RecruitmentPresenter::create_recruitment_buttons(&battle_style.display_name, &element_emojis);
        let buttons: Vec<_> = button_components.iter().map(to_create_action_row).collect();

        // 6. Discordメッセージを投稿（マルチ募集チャンネルに投稿）
        let channel_id = poise::serenity_prelude::ChannelId::new(recruitment_channel_id as u64);
        let message = channel_id
            .send_message(
                http,
                CreateMessage::new()
                    .content(message_content)
                    .embed(to_create_embed(&embed_content))
                    .components(buttons),
            )
            .await?;

        let message_id = message.id.get();

        debug!(
            message_id = %message_id,
            "Discordメッセージを投稿しました"
        );

        // 7. battle_recruitmentsに保存
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        let recruitment = battle_recruitment_repo
            .create_with_txn(
                txn,
                crate::repository::CreateBattleRecruitmentParams {
                    guild_id: params.guild_id as u64,
                    channel_id: recruitment_channel_id as u64,
                    message_id,
                    quest_id: quest.id,
                    battle_style_id: quest.default_battle_style_id,
                    quest_start_at: params.quest_start_at,
                },
            )
            .await?;

        info!(
            recruitment_id = recruitment.id,
            "マッチング募集をデータベースに登録しました"
        );

        // 8. 出発時刻の通知を登録（5分前とちょうどの時刻）
        debug!(
            quest_start_at = %params.quest_start_at,
            "募集の出発通知を登録します"
        );

        let notification_management_service = NotificationManagementService::new();
        notification_management_service
            .create_recruitment_departure_notification(
                txn,
                params.quest_start_at,
                params.guild_id,
                recruitment_channel_id,
                recruitment.id,
            )
            .await?;

        info!(
            recruitment_id = recruitment.id,
            "マッチング募集の出発通知を登録しました"
        );

        Ok(recruitment.id)
    }
}
