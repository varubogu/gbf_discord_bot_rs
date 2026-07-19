use crate::gateway::DiscordGuildGateway;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::presenter::RecruitmentPresenter;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::new;
use crate::services::recruitment::recruit_datetime_service::parse_quest_departure_datetime;
use crate::services::recruitment::recruitment_update_service::RecruitmentUpdateService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{
    DismissalManagementService, NotificationManagementService,
    RecruitmentMessageDeletionScheduleService,
};
use crate::services::timezone_service::TimezoneService;
use crate::services::unified_datetime_parser::ParsedDismissalTime;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types;
use crate::types::AppState;
use crate::types::discord::{ActionRowContent, DiscordMessageId, EmbedContent};
use sea_orm::TransactionTrait;
use tracing::{debug, info, instrument};

/// 募集作成結果（events層でのメッセージ送信用）
#[derive(Debug, Clone)]
pub struct RecruitmentResult {
    /// DB保存後の募集ID
    pub recruitment_id: i32,
    /// メッセージ本文
    pub message_content: String,
    /// Embed内容
    pub embed_content: EmbedContent,
    /// ボタン/セレクトメニュー（ボタン版のみ使用）
    pub components: Vec<ActionRowContent>,
    /// リアクション絵文字（リアクション版のみ使用）
    pub reaction_emojis: Vec<String>,
}

/// 新しい募集を開始する
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `quest_alias` - クエスト名またはエイリアス
/// * `battle_style_id` - 攻略方法ID（オプション）
/// * `event_date_input` - 開催日時入力（オプション）
/// * `use_buttons` - ボタンを使用する場合は true、リアクションを使用する場合は false
/// * `dismissal_times` - 解散時刻（オプション）
/// * `host_discord_user_id` - 募集作成者のDiscordユーザーID
///
/// # 戻り値
/// RecruitmentResult - 募集ID、表示用メッセージ、Embed、コンポーネント等
/// 注: message_idは0で仮保存されるため、events層でメッセージ送信後に`update_message_id`を呼び出すこと
#[allow(clippy::too_many_arguments)]
#[instrument(level = "debug", skip(app_state, gateway))]
pub async fn new_recruitment<G>(
    app_state: &AppState,
    gateway: &G,
    guild_id: u64,
    channel_id: u64,
    quest_alias: &str,
    battle_style_id: Option<i32>,
    event_date_input: Option<String>,
    use_buttons: bool,
    dismissal_times: Option<String>,
    host_discord_user_id: u64,
) -> types::Result<RecruitmentResult>
where
    G: DiscordGuildGateway + Sync,
{
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Repositoryの取得
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;

        // タイムゾーンを取得
        let timezone_repo = app_state.repositories.guild_settings;
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service
            .get_guild_timezone_with_txn(&txn, guild_id as i64)
            .await?;
        let event_date = if let Some(input) = event_date_input.as_deref() {
            Some(parse_quest_departure_datetime(input, timezone)?)
        } else {
            None
        };

        // 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = app_state.repositories.guild_environment;
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service
            .get_element_emojis(&txn, gateway, guild_id as i64)
            .await?;

        // 1. 募集データ作成（Repository DI）
        let quest_repository = app_state.repositories.quest;
        let battle_style_repository = app_state.repositories.battle_style;

        let message_service = app_state.message_service();
        let mut recruitment_data = new::create_recruitment_data(
            &txn,
            &quest_repository,
            &battle_style_repository,
            &element_emojis,
            message_service,
            new::RecruitmentParams {
                quest_name_or_alias: quest_alias,
                battle_style_id,
                channel_id,
                guild_id,
                event_date,
                timezone,
            },
        )
        .await?;

        // 1.5. ロールメンションを取得してメッセージの先頭に追加
        let all_roles_repo = app_state.repositories.all_recruitment_notification_roles;
        let quest_roles_repo = app_state.repositories.quest_recruitment_notification_roles;
        let role_service = RoleNotificationService::new(all_roles_repo, quest_roles_repo);
        let role_mentions = role_service
            .get_role_mentions(&txn, guild_id as i64, recruitment_data.quest.id)
            .await?;

        // 1.7. 解散時刻をパースしてメッセージに含める（指定されている場合）
        let parsed_dismissal_times = if let Some(ref dismissal_times_str) = dismissal_times {
            debug!(
                dismissal_times = %dismissal_times_str,
                "解散時刻のパースを開始します"
            );

            // 解散時刻をパース（統一パーサーを使用）
            let options =
                DateTimeParseOptions::for_dismissal_time(timezone, recruitment_data.expiry_date);
            let parsed_results = parse_datetime(dismissal_times_str, &options)?;

            // 元の入力値を分割（トリムして空文字除去）
            let input_values: Vec<&str> = dismissal_times_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            // ParsedDateTime を ParsedDismissalTime に変換
            let parsed_dismissal_times: Vec<ParsedDismissalTime> = parsed_results
                .iter()
                .enumerate()
                .map(|(idx, result)| {
                    let input_value = input_values.get(idx).unwrap_or(&"").to_string();
                    match result {
                        ParsedDateTime::Absolute(datetime) => Ok(ParsedDismissalTime::Absolute {
                            input_value,
                            datetime: *datetime,
                        }),
                        ParsedDateTime::Relative { days, hours, minutes } => {
                            Ok(ParsedDismissalTime::Relative {
                            input_value,
                            days: *days,
                            hours: *hours,
                            minutes: *minutes,
                        })
                        }
                        ParsedDateTime::Time(_) => Err(types::AppError::Business {
                            message: "解散時刻の解析結果が不正です。運用管理者に連絡してください。"
                                .to_string(),
                        }),
                    }
                })
                .collect::<types::Result<Vec<_>>>()?;

            // 解散時刻を含むメッセージを生成
            let message_service = app_state.message_service();
            let message_content_with_dismissal = new::create_message_content(
                &txn,
                message_service,
                new::MessageContentParams {
                    quest_name: &recruitment_data.quest.name,
                    battle_style_name: &recruitment_data.battle_style_name,
                    expiry_date: &recruitment_data.expiry_date,
                    timezone,
                    guild_id: Some(guild_id as i64),
                    dismissal_times: Some(&parsed_dismissal_times),
                },
            )
            .await?;

            // RecruitmentData のメッセージ内容を解散時刻付きで更新
            recruitment_data.message_content = message_content_with_dismissal;

            Some(parsed_dismissal_times)
        } else {
            None
        };

        // 1.8. ロールメンションがある場合は先頭に追加
        if !role_mentions.is_empty() {
            debug!(role_mentions = %role_mentions, "ロールメンションを募集メッセージの先頭に追加します");
            recruitment_data.message_content = format!("{}\n{}", role_mentions, recruitment_data.message_content);
            info!("ロールメンションを募集メッセージに追加しました");
        }

        // 2. データ保存（message_id=0で仮保存、events層でメッセージ送信後に更新）
        let recruitment = new::save_recruitment(&txn, &battle_recruitment_repo, &recruitment_data, 0, host_discord_user_id).await?;

        // 4. 出発時刻の通知を登録（5分前とちょうどの時刻）
        debug!(
            expiry_date = %recruitment_data.expiry_date,
            "募集の出発通知を登録します"
        );

        let notification_management_service = NotificationManagementService::new(
            app_state.repositories.notification,
            app_state.repositories.notification_rel_battle_recruitment,
            app_state.repositories.scheduled_task,
        );
        notification_management_service
            .create_recruitment_departure_notification(
                &txn,
                recruitment_data.expiry_date,
                guild_id as i64,
                channel_id as i64,
                recruitment.id,
            )
            .await?;

        // 5. 募集投稿削除タスクを登録
        let message_deletion_schedule_service = RecruitmentMessageDeletionScheduleService::new(
            app_state.repositories.guild_environment,
            app_state.repositories.environment,
            app_state.repositories.scheduled_task,
            app_state
                .repositories
                .scheduled_task_recruitment_message_deletion,
        );
        message_deletion_schedule_service
            .create_for_recruitment(
                &txn,
                guild_id as i64,
                channel_id as i64,
                recruitment.id,
                recruitment_data.expiry_date,
            )
            .await?;

        // 6. 解散時刻を登録（指定されている場合）
        if let Some(parsed_dismissal_times) = parsed_dismissal_times {
            debug!(
                recruitment_id = recruitment.id,
                dismissal_count = parsed_dismissal_times.len(),
                "解散時刻を登録します"
            );

            // 解散時刻を登録
            let dismissal_service = DismissalManagementService::new(
                app_state.repositories.battle_recruitment_dismissal,
                app_state.repositories.scheduled_task,
                app_state.repositories.scheduled_task_dismissal,
            );
            dismissal_service
                .create_recruitment_dismissals(
                    &txn,
                    recruitment.id,
                    parsed_dismissal_times,
                    recruitment_data.expiry_date,
                    guild_id as i64,
                    channel_id as i64,
                )
                .await?;

            info!(
                recruitment_id = recruitment.id,
                "解散時刻を登録しました"
            );
        }

        // 3. 表示用データを準備
        let (embed_content, components) = if use_buttons {
            create_v2_recruitment_embed_and_components(
                &recruitment_data.battle_style_name,
                &recruitment_data.element_emojis,
            )
        } else {
            // リアクション版用のEmbed（コンポーネントなし）
            (recruitment_data.embed_content.clone(), vec![])
        };

        Ok(RecruitmentResult {
            recruitment_id: recruitment.id,
            message_content: recruitment_data.message_content.clone(),
            embed_content,
            components,
            reaction_emojis: if use_buttons {
                vec![]
            } else {
                recruitment_data.reaction_emojis.clone()
            },
        })
    }
    .await;

    match result {
        Ok(recruitment_result) => {
            txn.commit().await?;
            Ok(recruitment_result)
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}

/// ボタン版募集の表示内容を組み立てる。
///
/// UIモデルへの変換はFacadeからPresenterへ委譲し、Service層を表示依存から分離する。
fn create_v2_recruitment_embed_and_components(
    battle_style_name: &str,
    element_emojis: &crate::services::guild_environment_service::ElementEmojis,
) -> (EmbedContent, Vec<ActionRowContent>) {
    let initial_text =
        RecruitmentPresenter::create_initial_participants_text(battle_style_name, element_emojis);
    let components = if battle_style_name == "6属性" {
        RecruitmentPresenter::create_six_element_full_components(element_emojis)
    } else {
        RecruitmentPresenter::create_recruitment_buttons(battle_style_name, element_emojis)
    };
    let embed = RecruitmentPresenter::create_participants_embed(&initial_text, Some(0));

    (embed, components)
}

/// メッセージ送信後にmessage_idを更新する
///
/// events層でメッセージ送信後に呼び出し、DBのmessage_idを更新する
#[instrument(level = "debug", skip(app_state))]
pub async fn update_message_id(
    app_state: &AppState,
    guild_id: u64,
    recruitment_id: i32,
    message_id: u64,
) -> types::Result<()> {
    info!(
        guild_id = guild_id,
        recruitment_id = recruitment_id,
        message_id = message_id,
        "募集のmessage_idを更新します"
    );

    let db = app_state.guild_db();
    let txn = db.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let update_service = RecruitmentUpdateService::new(app_state.repositories.battle_recruitments);
    let result = update_service
        .update_message_id(&txn, recruitment_id, DiscordMessageId::new(message_id))
        .await;

    match result {
        Ok(()) => {
            txn.commit().await?;
        }
        Err(e) => {
            txn.rollback().await?;
            return Err(e);
        }
    }

    info!(
        guild_id = guild_id,
        recruitment_id = recruitment_id,
        message_id = message_id,
        "message_idの更新が完了しました"
    );
    Ok(())
}
