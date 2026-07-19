use super::shared_presentation::build_v2_recruitment_embed_and_components;
use crate::gateway::DiscordGateway;
use crate::models::entities::worker::quest_matching_users;
use crate::presenter::NotificationPresenter;
use crate::repository::auto_recruitment::{
    AutoRecruitmentMatchRuleQuotaRepository, AutoRecruitmentMatchRuleRepository,
    AutoRecruitmentParticipantRepository, AutoRecruitmentRepository, QuestMatchingRepository,
    QuestMatchingUserRepository, UserDesiredQuestRepository,
};
use crate::repository::schedule::{
    BattleRecruitmentDismissalRepository, BattleRecruitmentScheduleDismissalRepository,
    NotificationRelBattleRecruitmentRepository, NotificationRepository,
    ScheduledTaskDismissalRepository, ScheduledTaskRepository,
};
use crate::repository::{
    AllRecruitmentNotificationRolesRepository, BattleRecruitmentsRepository, BattleStyleRepository,
    GuildChannelRepository, GuildEnvironmentRepository, GuildMessageTextRepository,
    GuildSettingsRepository, MessageTextRepository, QuestRecruitmentNotificationRolesRepository,
    QuestRepository,
};
use crate::services::auto_recruitment::PeriodicMatchingService;
use crate::services::recruitment::recruitment_creation_service::{
    CreatedMatchingRecruitmentInfo, MatchingRecruitmentParams, RecruitmentCreationService,
};
use crate::services::schedule::RecruitmentMessageDeletionScheduler;
use crate::services::schedule::auto_matching_dispatch_support_service::AutoMatchingDispatchSupportService;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, MessageContent};
use crate::types::{AppError, Result};
use chrono::{Datelike, TimeZone, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::collections::HashMap;
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

/// 自動マッチングタスクを実行する
///
/// 元は`AutoMatchingTaskExecutor`（service層）にあったロジック。
/// Facadeが複数serviceを合成してユースケースを実行する責務に合わせてここへ統合した。
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_auto_matching_dispatch<
    G,
    ST,
    ARR,
    QMR,
    QMUR,
    Q,
    APR,
    UDR,
    RMR,
    RMQ,
    GC,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    dispatch_support: &AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>,
    matching_service: &PeriodicMatchingService<APR, UDR, QMR, QMUR, Q, RMR, RMQ>,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    task_id: i32,
) -> Result<AutoMatchingResult>
where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository + Clone,
    APR: AutoRecruitmentParticipantRepository,
    UDR: UserDesiredQuestRepository,
    RMR: AutoRecruitmentMatchRuleRepository,
    RMQ: AutoRecruitmentMatchRuleQuotaRepository,
    GC: GuildChannelRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    info!(task_id, "自動マッチングタスク実行開始");

    // タスクが削除されていないか、既に実行済みでないかを確認
    let _task = match dispatch_support.find_task(txn, task_id).await? {
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

    // マッチング処理を実行
    let matchings = matching_service.process_matching(txn).await?;

    let matched_groups = matchings.len();

    // マッチング通知を送信し、マルチ募集を作成
    if !matchings.is_empty() {
        send_match_notifications_and_create_recruitments(
            txn,
            db_conn,
            gateway,
            dispatch_support,
            recruitment_creation_service,
            &matchings,
        )
        .await?;
    }

    // タスクを正常終了にマーク
    dispatch_support.mark_succeeded(txn, task_id).await?;

    // 次回タスクを作成（10秒後）
    let next_task_id = dispatch_support.register_next_scheduled_task(txn).await?;

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
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
async fn send_match_notifications_and_create_recruitments<
    G,
    ST,
    ARR,
    QMR,
    QMUR,
    Q,
    GC,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    dispatch_support: &AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    matchings: &[crate::models::entities::worker::quest_matchings::Model],
) -> Result<()>
where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository + Clone,
    GC: GuildChannelRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
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
        let auto_recruitment = match dispatch_support
            .find_auto_recruitment_by_guild(txn, guild_id)
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
            process_single_matching(
                txn,
                db_conn,
                gateway,
                dispatch_support,
                recruitment_creation_service,
                guild_id,
                matching_channel_id,
                matching,
            )
            .await?;
        }
    }

    Ok(())
}

/// 単一のマッチングに対して、通知送信・マルチ募集作成・募集IDの紐付けを行う
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
async fn process_single_matching<
    G,
    ST,
    ARR,
    QMR,
    QMUR,
    Q,
    GC,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    dispatch_support: &AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    guild_id: i64,
    matching_channel_id: u64,
    matching: &crate::models::entities::worker::quest_matchings::Model,
) -> Result<()>
where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository + Clone,
    GC: GuildChannelRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    // クエスト情報を取得
    let quest = match dispatch_support.find_quest(txn, matching.quest_id).await? {
        Some(q) => q,
        None => {
            warn!(
                guild_id,
                quest_id = matching.quest_id,
                "クエストが見つかりません"
            );
            return Ok(());
        }
    };

    // 参加ユーザーを取得
    let users = dispatch_support
        .find_active_matching_users(txn, guild_id, matching.id)
        .await?;

    if users.is_empty() {
        return Ok(());
    }

    let user_ids: Vec<u64> = users.iter().map(|u| u.user_id as u64).collect();

    // 通知を送信（Gateway経由）
    let notification_message_id = match send_auto_matching_notification(
        gateway,
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
        Ok(message_id) => Some(message_id),
        Err(e) => {
            error!(
                error = %e,
                guild_id,
                matching_id = %matching.id,
                "マッチング通知の送信に失敗しました"
            );
            // 通知失敗しても募集作成は試みる
            None
        }
    };

    // 出発時刻を計算
    let quest_start_at = calculate_quest_start_at(
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
        return Ok(());
    }

    // マルチ募集を作成
    let params = MatchingRecruitmentParams {
        guild_id,
        quest_id: matching.quest_id,
        quest_start_at,
        participant_user_ids: user_ids,
    };

    match create_and_send_matching_recruitment(
        txn,
        db_conn,
        gateway,
        recruitment_creation_service,
        &params,
    )
    .await
    {
        Ok(recruitment) => {
            info!(
                guild_id,
                matching_id = %matching.id,
                recruitment_id = recruitment.recruitment_id,
                "マルチ募集を作成しました"
            );

            link_matching_recruitment(
                txn,
                gateway,
                dispatch_support,
                guild_id,
                matching_channel_id,
                matching,
                &quest.name,
                &users,
                notification_message_id,
                &recruitment,
            )
            .await;
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

    Ok(())
}

/// マッチングへの募集ID設定と、通知メッセージへの募集リンク追記を行う
///
/// いずれも失敗時はログのみでベストエフォート（募集自体は既に作成済みのため）。
#[allow(clippy::too_many_arguments)]
async fn link_matching_recruitment<G, ST, ARR, QMR, QMUR, Q>(
    txn: &DatabaseTransaction,
    gateway: &G,
    dispatch_support: &AutoMatchingDispatchSupportService<ST, ARR, QMR, QMUR, Q>,
    guild_id: i64,
    matching_channel_id: u64,
    matching: &crate::models::entities::worker::quest_matchings::Model,
    quest_name: &str,
    users: &[quest_matching_users::Model],
    notification_message_id: Option<DiscordMessageId>,
    recruitment: &CreatedMatchingRecruitmentInfo,
) where
    G: DiscordGateway,
    ST: ScheduledTaskRepository,
    ARR: AutoRecruitmentRepository,
    QMR: QuestMatchingRepository,
    QMUR: QuestMatchingUserRepository,
    Q: QuestRepository + Clone,
{
    // マッチングに募集IDを設定
    if let Err(e) = dispatch_support
        .set_matching_recruitment_id(txn, guild_id, matching.id, recruitment.recruitment_id)
        .await
    {
        error!(
            error = %e,
            guild_id,
            matching_id = %matching.id,
            recruitment_id = recruitment.recruitment_id,
            "マッチングへの募集ID設定に失敗しました"
        );
    }

    if let Some(notification_message_id) = notification_message_id
        && let Err(e) = edit_auto_matching_notification_with_link(
            gateway,
            matching_channel_id,
            notification_message_id,
            guild_id,
            quest_name,
            matching.scheduled_month,
            matching.scheduled_day,
            matching.scheduled_hour,
            &users
                .iter()
                .map(|u| (u.user_id as u64, u.battle_style_id))
                .collect::<Vec<_>>(),
            recruitment,
        )
        .await
    {
        error!(
            error = %e,
            guild_id,
            matching_id = %matching.id,
            recruitment_id = recruitment.recruitment_id,
            "マッチング通知への募集リンク追記に失敗しました"
        );
    }
}

/// マッチングから募集の表示データを組み立て、Discordへ投稿し、DB保存する
///
/// UI組み立て（Presenter）とDiscord送信（Gateway）はFacade層の責務。
#[allow(clippy::type_complexity)]
async fn create_and_send_matching_recruitment<
    G,
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
    MDS,
>(
    txn: &DatabaseTransaction,
    db_conn: &DatabaseConnection,
    gateway: &G,
    recruitment_creation_service: &RecruitmentCreationService<
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
        MDS,
    >,
    params: &MatchingRecruitmentParams,
) -> Result<CreatedMatchingRecruitmentInfo>
where
    G: DiscordGateway,
    GC: GuildChannelRepository,
    Q: QuestRepository,
    BS: BattleStyleRepository,
    A: AllRecruitmentNotificationRolesRepository,
    QR: QuestRecruitmentNotificationRolesRepository,
    GE: GuildEnvironmentRepository,
    SD: BattleRecruitmentScheduleDismissalRepository,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
    NMN: NotificationRepository,
    NMR: NotificationRelBattleRecruitmentRepository,
    NMS: ScheduledTaskRepository,
    DR: BattleRecruitmentDismissalRepository,
    TR: ScheduledTaskRepository,
    TDR: ScheduledTaskDismissalRepository,
    GS: GuildSettingsRepository,
    BR: BattleRecruitmentsRepository,
    MDS: RecruitmentMessageDeletionScheduler,
{
    let prepared = recruitment_creation_service
        .prepare_recruitment_from_matching(txn, db_conn, gateway, params)
        .await?;

    let (embed_content, button_components) = build_v2_recruitment_embed_and_components(
        &prepared.battle_style_name,
        &prepared.element_emojis,
    );

    let channel_id = DiscordChannelId::new(prepared.recruitment_channel_id as u64);
    let domain_message_content = MessageContent::new()
        .with_text(&prepared.message_content)
        .with_embed(embed_content)
        .with_components(button_components);

    let sent_message_id = gateway
        .send_message(channel_id, domain_message_content)
        .await?;
    let message_id = sent_message_id.get();

    debug!(message_id = %message_id, "Discordメッセージを投稿しました");

    recruitment_creation_service
        .finalize_recruitment_from_matching(txn, params, prepared, message_id)
        .await
}

/// スケジュール情報から出発時刻を計算
fn calculate_quest_start_at(month: i32, day: i32, hour: i32) -> chrono::DateTime<Utc> {
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

/// 個別のマッチング通知を送信（Gateway経由）
#[allow(clippy::too_many_arguments)]
async fn send_auto_matching_notification<G: DiscordGateway>(
    gateway: &G,
    channel_id: u64,
    quest_name: &str,
    month: i32,
    day: i32,
    hour: i32,
    users: &[(u64, Option<i32>)],
) -> Result<DiscordMessageId> {
    let channel = DiscordChannelId::new(channel_id);
    let message_content = NotificationPresenter::create_auto_matching_notification(
        quest_name, month, day, hour, users, None,
    );

    let message_id = gateway
        .send_message(channel, message_content)
        .await
        .map_err(|e| AppError::Business {
            message: format!("マッチング通知の送信に失敗しました: {e}"),
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

    Ok(message_id)
}

/// 募集作成後にマッチング通知へジャンプリンクを追記
#[allow(clippy::too_many_arguments)]
async fn edit_auto_matching_notification_with_link<G: DiscordGateway>(
    gateway: &G,
    notification_channel_id: u64,
    notification_message_id: DiscordMessageId,
    guild_id: i64,
    quest_name: &str,
    month: i32,
    day: i32,
    hour: i32,
    users: &[(u64, Option<i32>)],
    recruitment: &CreatedMatchingRecruitmentInfo,
) -> Result<()> {
    let channel = DiscordChannelId::new(notification_channel_id);
    let recruitment_url = format!(
        "https://discord.com/channels/{guild_id}/{}/{}",
        recruitment.channel_id, recruitment.message_id
    );
    let message_content = NotificationPresenter::create_auto_matching_notification(
        quest_name,
        month,
        day,
        hour,
        users,
        Some(&recruitment_url),
    );

    gateway
        .edit_message(channel, notification_message_id, message_content)
        .await
        .map_err(|e| AppError::Business {
            message: format!("マッチング通知の編集に失敗しました: {e}"),
        })?;

    Ok(())
}
