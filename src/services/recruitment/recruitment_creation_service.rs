// Note: converters are no longer needed as we use domain types directly with Gateway
use crate::gateway::DiscordGateway;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::entities::master::channel_types::GuildChannelType;
use crate::presenter::RecruitmentPresenter;
use crate::repository::BattleRecruitmentsRepository;
use crate::repository::BattleStyleRepository;
use crate::repository::GuildChannelRepository;
use crate::repository::QuestRepository;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::message::MessageService;
use crate::services::recruitment::new::{MessageContentParams, create_message_content};
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{DismissalManagementService, NotificationManagementService};
use crate::services::timezone_service::TimezoneService;
use crate::services::unified_datetime_parser::ParsedDismissalTime;
use crate::types::Result;
use crate::types::discord::{
    DiscordChannelId, DiscordGuildId, DiscordMessageId, DiscordUserId, EmbedContent, MessageContent,
};
use chrono::{TimeZone, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
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
pub struct RecruitmentCreationService<
    GC,
    Q,
    BS,
    A,
    QR,
    GE,
    SD,
    GM,
    MT,
    NMN,
    NMR,
    NMS,
    DR,
    TR,
    TDR,
    GS,
    BR,
> where
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
{
    guild_channel_repo: GC,
    quest_repo: Q,
    battle_style_repo: BS,
    role_service: RoleNotificationService<A, QR>,
    timezone_service: TimezoneService<GS>,
    guild_env_service: GuildEnvironmentService<GE>,
    schedule_dismissal_repo: SD,
    message_service: MessageService<GM, MT>,
    notification_management_service: NotificationManagementService<NMN, NMR, NMS>,
    dismissal_service: DismissalManagementService<DR, TR, TDR>,
    battle_recruitment_repo: BR,
}

impl<GC, Q, BS, A, QR, GE, SD, GM, MT, NMN, NMR, NMS, DR, TR, TDR, GS, BR>
    RecruitmentCreationService<GC, Q, BS, A, QR, GE, SD, GM, MT, NMN, NMR, NMS, DR, TR, TDR, GS, BR>
where
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: crate::repository::AllRecruitmentNotificationRolesRepository,
    QR: crate::repository::QuestRecruitmentNotificationRolesRepository,
    GE: crate::repository::GuildEnvironmentRepository,
    SD: crate::repository::schedule::BattleRecruitmentScheduleDismissalRepository,
    GM: crate::repository::GuildMessageTextRepository,
    MT: crate::repository::MessageTextRepository,
    NMN: crate::repository::schedule::NotificationRepository,
    NMR: crate::repository::schedule::NotificationRelBattleRecruitmentRepository,
    NMS: crate::repository::schedule::ScheduledTaskRepository,
    DR: crate::repository::schedule::BattleRecruitmentDismissalRepository,
    TR: crate::repository::schedule::ScheduledTaskRepository,
    TDR: crate::repository::schedule::ScheduledTaskDismissalRepository,
    GS: crate::repository::GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guild_channel_repo: GC,
        quest_repo: Q,
        battle_style_repo: BS,
        role_service: RoleNotificationService<A, QR>,
        timezone_service: TimezoneService<GS>,
        guild_env_service: GuildEnvironmentService<GE>,
        schedule_dismissal_repo: SD,
        message_service: MessageService<GM, MT>,
        notification_management_service: NotificationManagementService<NMN, NMR, NMS>,
        dismissal_service: DismissalManagementService<DR, TR, TDR>,
        battle_recruitment_repo: BR,
    ) -> Self {
        Self {
            guild_channel_repo,
            quest_repo,
            battle_style_repo,
            role_service,
            timezone_service,
            guild_env_service,
            schedule_dismissal_repo,
            message_service,
            notification_management_service,
            dismissal_service,
            battle_recruitment_repo,
        }
    }

    /// スケジュールから募集を作成
    /// 定期募集スケジュールに基づいて実際の募集を作成
    pub async fn create_recruitment_from_schedule<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &G,
        calculated_time: &crate::services::schedule::CalculatedRecruitmentTime,
    ) -> Result<()> {
        debug!(
            schedule_id = calculated_time.schedule_id,
            quest_id = calculated_time.quest_id,
            "スケジュールから募集を作成します"
        );

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(txn, calculated_time.guild_id).await?;

        // 0. マルチ募集チャンネルを取得
        let guild_channel = self
            .guild_channel_repo
            .get_by_guild_and_type_with_txn(
                txn,
                calculated_time.guild_id,
                GuildChannelType::MultiRecruitment.as_i32(),
            )
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
        let quest = self
            .quest_repo
            .get_by_target_id(db_conn, calculated_time.quest_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    calculated_time.quest_id
                ))
            })?;

        let battle_style = self
            .battle_style_repo
            .get_by_id(db_conn, calculated_time.battle_style_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "攻略方法ID {} が見つかりませんでした",
                    calculated_time.battle_style_id
                ))
            })?;

        let timezone = self
            .timezone_service
            .get_guild_timezone_with_txn(txn, calculated_time.guild_id)
            .await?;

        // 2. ロールメンションを取得
        let role_mentions = self
            .role_service
            .get_role_mentions(txn, calculated_time.guild_id, quest.id)
            .await?;

        // 2.5. 定期募集スケジュールの解散時刻を取得
        let schedule_dismissals = self
            .schedule_dismissal_repo
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
            &self.message_service,
            MessageContentParams {
                quest_name: &quest.name,
                battle_style_name: &battle_style.display_name,
                expiry_date: &calculated_time.quest_start_at,
                timezone,
                guild_id: Some(calculated_time.guild_id),
                dismissal_times: dismissal_times_option,
            },
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

        // 3.5. 属性絵文字を取得（ギルド固有設定 or デフォルト値）（Gateway経由）
        let element_emojis = self
            .guild_env_service
            .get_element_emojis(txn, gateway, calculated_time.guild_id)
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

        // 5. ボタンを作成（Presenterのドメインモデル）
        let button_components = RecruitmentPresenter::create_recruitment_buttons(
            &battle_style.display_name,
            &element_emojis,
        );

        // 6. Discordメッセージを投稿（マルチ募集チャンネルに投稿）（Gateway経由）
        let channel_id = DiscordChannelId::new(recruitment_channel_id as u64);
        let domain_message_content = MessageContent::new()
            .with_text(&message_content)
            .with_embed(embed_content)
            .with_components(button_components);

        let sent_message_id = gateway
            .send_message(channel_id, domain_message_content)
            .await?;

        let message_id = sent_message_id.get();

        debug!(
            message_id = %message_id,
            "Discordメッセージを投稿しました"
        );

        // 7. battle_recruitmentsに保存
        // i64/u64をドメイン型に変換してRepositoryに渡す
        let recruitment = self
            .battle_recruitment_repo
            .create_with_txn(
                txn,
                crate::repository::CreateBattleRecruitmentParams {
                    guild_id: DiscordGuildId::new(calculated_time.guild_id as u64),
                    channel_id: DiscordChannelId::new(recruitment_channel_id as u64),
                    message_id: DiscordMessageId::new(message_id),
                    quest_id: quest.id,
                    battle_style_id: calculated_time.battle_style_id,
                    quest_start_at: calculated_time.quest_start_at,
                    host_discord_user_id: DiscordUserId::new(0), // 自動作成のため作成者不明
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

        self.notification_management_service
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

            self.dismissal_service
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
    pub async fn create_recruitment_from_matching<G: DiscordGateway>(
        &self,
        txn: &DatabaseTransaction,
        db_conn: &DatabaseConnection,
        gateway: &G,
        params: &MatchingRecruitmentParams,
    ) -> Result<i32> {
        debug!(
            guild_id = params.guild_id,
            quest_id = params.quest_id,
            "マッチングから募集を作成します"
        );

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(txn, params.guild_id).await?;

        // 0. マルチ募集チャンネルを取得
        let guild_channel = self
            .guild_channel_repo
            .get_by_guild_and_type_with_txn(
                txn,
                params.guild_id,
                GuildChannelType::MultiRecruitment.as_i32(),
            )
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
        let quest = self
            .quest_repo
            .get_by_target_id(db_conn, params.quest_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    params.quest_id
                ))
            })?;

        // クエストのデフォルト攻略方法を使用
        let battle_style = self
            .battle_style_repo
            .get_by_id(db_conn, quest.default_battle_style_id)
            .await?
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "攻略方法ID {} が見つかりませんでした",
                    quest.default_battle_style_id
                ))
            })?;

        let timezone = self
            .timezone_service
            .get_guild_timezone_with_txn(txn, params.guild_id)
            .await?;

        // 2. ロールメンションを取得
        let role_mentions = self
            .role_service
            .get_role_mentions(txn, params.guild_id, quest.id)
            .await?;

        // 3. メッセージ内容を作成（マッチングでは解散時刻なし）
        let mut message_content = create_message_content(
            txn,
            &self.message_service,
            MessageContentParams {
                quest_name: &quest.name,
                battle_style_name: &battle_style.display_name,
                expiry_date: &params.quest_start_at,
                timezone,
                guild_id: Some(params.guild_id),
                dismissal_times: None, // 解散時刻なし
            },
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
                .map(|user_id| format!("<@{user_id}>"))
                .collect();
            message_content = format!("{}\n{message_content}", user_mentions.join(" "));
        }

        // 3.5. 属性絵文字を取得（ギルド固有設定 or デフォルト値）（Gateway経由）
        let element_emojis = self
            .guild_env_service
            .get_element_emojis(txn, gateway, params.guild_id)
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

        // 5. ボタンを作成（Presenterのドメインモデル）
        let button_components = RecruitmentPresenter::create_recruitment_buttons(
            &battle_style.display_name,
            &element_emojis,
        );

        // 6. Discordメッセージを投稿（マルチ募集チャンネルに投稿）（Gateway経由）
        let channel_id = DiscordChannelId::new(recruitment_channel_id as u64);
        let domain_message_content = MessageContent::new()
            .with_text(&message_content)
            .with_embed(embed_content)
            .with_components(button_components);

        let sent_message_id = gateway
            .send_message(channel_id, domain_message_content)
            .await?;

        let message_id = sent_message_id.get();

        debug!(
            message_id = %message_id,
            "Discordメッセージを投稿しました"
        );

        // 7. battle_recruitmentsに保存
        // i64/u64をドメイン型に変換してRepositoryに渡す
        let recruitment = self
            .battle_recruitment_repo
            .create_with_txn(
                txn,
                crate::repository::CreateBattleRecruitmentParams {
                    guild_id: DiscordGuildId::new(params.guild_id as u64),
                    channel_id: DiscordChannelId::new(recruitment_channel_id as u64),
                    message_id: DiscordMessageId::new(message_id),
                    quest_id: quest.id,
                    battle_style_id: quest.default_battle_style_id,
                    quest_start_at: params.quest_start_at,
                    host_discord_user_id: DiscordUserId::new(0), // 自動作成のため作成者不明
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

        self.notification_management_service
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
