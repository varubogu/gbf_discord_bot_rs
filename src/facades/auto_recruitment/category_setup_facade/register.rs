use super::common::build_setup_service;
use super::messages::{
    send_matching_channel_message, send_quest_channel_messages, send_time_selection_message,
};
use crate::gateway::{DiscordChannelGateway, DiscordMessageGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::entities::guild_master::auto_recruitments;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentQuestMessageRepository,
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::CategorySetupService;
use crate::types::discord::{
    ChannelCreateParams, ChannelEditParams, DiscordChannelId, DiscordGuildId,
};
use crate::types::{AppError, AppState, Result};
use crate::utils::datetime_display::format_date_channel_name_ja;
use chrono::{Datelike, Duration, Utc};
use sea_orm::TransactionTrait;
use tracing::{debug, error, info, instrument};

/// カテゴリ登録結果
pub struct CategoryRegistrationResult {
    /// カテゴリチャンネルID
    pub category_id: u64,
    /// 作成された日時チャンネル数
    pub channel_count: usize,
}

/// カテゴリを自動募集に登録
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `category_id` - カテゴリチャンネルID
/// * `days` - 募集日数（2-7日）
/// * `matching_channel_id` - マッチング通知チャンネルID（省略可能）
/// * `quest_channel_id` - クエスト選択チャンネルID（省略可能）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn register_category<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    category_id: u64,
    days: i32,
    matching_channel_id: Option<u64>,
    quest_channel_id: Option<u64>,
) -> Result<CategoryRegistrationResult>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    info!(guild_id, category_id, days, "自動募集カテゴリを登録します");

    // 日数の検証
    if !(2..=7).contains(&days) {
        return Err(AppError::Business {
            message: "募集日数は2〜7日の範囲で指定してください".to_string(),
        });
    }

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let setup_service = build_setup_service(app_state);

        // 既存の登録をチェック
        setup_service
            .ensure_not_registered(&txn, guild_id as i64)
            .await?;

        // チャンネル順序: マッチング(0) → 日付昇順(1〜days) → クエスト(days+1)
        // マッチング/クエストチャンネルを準備し、クエストメッセージ送信とレコード作成まで行う
        let (auto_recruitment, final_quest_channel_id) = provision_and_record(
            gateway,
            &setup_service,
            &txn,
            guild_id,
            category_id,
            days,
            matching_channel_id,
            quest_channel_id,
        )
        .await?;

        info!(
            guild_id = auto_recruitment.guild_id,
            "自動募集設定を登録しました"
        );

        // 日時チャンネルを作成（position 1〜days）
        let created_channels =
            create_date_channels(gateway, &setup_service, &txn, guild_id, category_id, days)
                .await?;

        // クエストチャンネルの位置を日付チャンネルの後に設定（position days+1）
        let quest_channel = DiscordChannelId::new(final_quest_channel_id);
        let _ = gateway
            .edit_channel(
                quest_channel,
                ChannelEditParams::new().with_position((days + 1) as u16),
            )
            .await;

        // ローテーションタスクと自動マッチングタスクを初期登録
        setup_service.ensure_initial_rotation_task(&txn).await?;
        setup_service
            .ensure_initial_auto_matching_task(&txn)
            .await?;

        Ok(CategoryRegistrationResult {
            category_id,
            channel_count: created_channels,
        })
    }
    .await;

    match result {
        Ok(res) => {
            txn.commit().await?;
            info!(
                guild_id,
                category_id,
                channel_count = res.channel_count,
                "自動募集カテゴリを登録しました"
            );
            Ok(res)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, category_id, "自動募集カテゴリの登録に失敗しました");
            Err(e)
        }
    }
}

/// マッチングチャンネルを準備する（指定チャンネルへ送信 or 新規作成）
///
/// # 戻り値
/// `(チャンネルID, Bot作成フラグ, メッセージID)`
async fn provision_matching_channel<G>(
    gateway: &G,
    discord_guild_id: DiscordGuildId,
    category_id: u64,
    matching_channel_id: Option<u64>,
) -> Result<(u64, bool, Option<u64>)>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    if let Some(ch_id) = matching_channel_id {
        // 指定されたチャンネルにメッセージを送信し、位置を調整
        let channel_id = DiscordChannelId::new(ch_id);
        let msg_id = send_matching_channel_message(gateway, channel_id).await?;
        // 指定チャンネルの位置を0に設定
        let _ = gateway
            .edit_channel(channel_id, ChannelEditParams::new().with_position(0))
            .await;
        Ok((ch_id, false, Some(msg_id.get())))
    } else {
        // チャンネルを新規作成（position 0）
        let channel_params = ChannelCreateParams::text("マッチング")
            .with_parent(DiscordChannelId::new(category_id))
            .with_position(0);

        let new_channel_id = gateway
            .create_channel(discord_guild_id, channel_params)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id = discord_guild_id.get(), "マッチングチャンネルの作成に失敗しました");
                AppError::ChannelCreationFailed
            })?;

        let msg_id = send_matching_channel_message(gateway, new_channel_id).await?;
        Ok((new_channel_id.get(), true, Some(msg_id.get())))
    }
}

/// クエストチャンネルを準備する（指定チャンネル利用 or 新規作成）
///
/// # 戻り値
/// `(チャンネルID, Bot作成フラグ)`
async fn provision_quest_channel<G>(
    gateway: &G,
    discord_guild_id: DiscordGuildId,
    category_id: u64,
    quest_channel_id: Option<u64>,
) -> Result<(u64, bool)>
where
    G: DiscordChannelGateway + Sync,
{
    if let Some(ch_id) = quest_channel_id {
        Ok((ch_id, false))
    } else {
        // チャンネルを新規作成（位置は後で設定）
        let channel_params = ChannelCreateParams::text("クエスト選択")
            .with_parent(DiscordChannelId::new(category_id));

        let new_channel_id = gateway
            .create_channel(discord_guild_id, channel_params)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id = discord_guild_id.get(), "クエストチャンネルの作成に失敗しました");
                AppError::ChannelCreationFailed
            })?;

        Ok((new_channel_id.get(), true))
    }
}

/// マッチング/クエストチャンネルを準備し、クエストメッセージ送信とauto_recruitmentsレコード作成までを行う
///
/// # 戻り値
/// `(作成されたauto_recruitmentsレコード, クエストチャンネルID)`
#[allow(clippy::too_many_arguments)]
async fn provision_and_record<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
    category_id: u64,
    days: i32,
    matching_channel_id: Option<u64>,
    quest_channel_id: Option<u64>,
) -> Result<(auto_recruitments::Model, u64)>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
    AR: AutoRecruitmentRepository,
    AC: AutoRecruitmentChannelRepository,
    Q: QuestRepository,
    AQM: AutoRecruitmentQuestMessageRepository,
    QMU: QuestMatchingUserRepository,
    QM: QuestMatchingRepository,
    ST: ScheduledTaskRepository,
{
    let discord_guild_id = DiscordGuildId::new(guild_id);

    // マッチングチャンネルの処理（position 0）
    let (final_matching_channel_id, matching_is_bot_created, matching_message_id) =
        provision_matching_channel(gateway, discord_guild_id, category_id, matching_channel_id)
            .await?;

    // クエスト一覧を取得（有効なクエストのみ）
    let enabled_quests = setup_service
        .get_enabled_quests(txn, guild_id as i64)
        .await?;

    // クエストチャンネルの処理（position days+1、日付チャンネル作成後に位置を設定）
    let (final_quest_channel_id, quest_is_bot_created) =
        provision_quest_channel(gateway, discord_guild_id, category_id, quest_channel_id).await?;

    // 1クエスト1メッセージ形式でメッセージを送信し、メッセージIDを保存
    let quest_channel_id_domain = DiscordChannelId::new(final_quest_channel_id);
    let quest_message_mappings =
        send_quest_channel_messages(gateway, quest_channel_id_domain, guild_id, &enabled_quests)
            .await?;
    for (quest_id, sent_message_id) in quest_message_mappings {
        setup_service
            .upsert_quest_message(txn, guild_id as i64, quest_id, sent_message_id)
            .await?;
    }

    // auto_recruitmentsテーブルに登録
    let auto_recruitment = setup_service
        .create_auto_recruitment(
            txn,
            guild_id as i64,
            category_id as i64,
            Some(final_matching_channel_id as i64),
            Some(final_quest_channel_id as i64),
            matching_is_bot_created,
            quest_is_bot_created,
            matching_message_id.map(|id| id as i64),
            days,
        )
        .await?;

    Ok((auto_recruitment, final_quest_channel_id))
}

/// 日時チャンネルを作成（position 1〜days）し、作成数を返す
#[allow(clippy::too_many_arguments)]
async fn create_date_channels<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
    category_id: u64,
    days: i32,
) -> Result<usize>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
    AR: AutoRecruitmentRepository,
    AC: AutoRecruitmentChannelRepository,
    Q: QuestRepository,
    AQM: AutoRecruitmentQuestMessageRepository,
    QMU: QuestMatchingUserRepository,
    QM: QuestMatchingRepository,
    ST: ScheduledTaskRepository,
{
    let now_utc = Utc::now();
    let now_jst = now_utc + Duration::hours(9);
    let today = now_jst.date_naive();
    let discord_guild_id = DiscordGuildId::new(guild_id);
    let mut created_channels = 0;

    for i in 0..days {
        let date = today + Duration::days(i as i64);
        let channel_name = format_date_channel_name_ja(date);
        let channel_position = (i + 1) as u16;

        let channel_params = ChannelCreateParams::text(&channel_name)
            .with_parent(DiscordChannelId::new(category_id))
            .with_position(channel_position);

        let new_channel_id = gateway
            .create_channel(discord_guild_id, channel_params)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, category_id, "チャンネルの作成に失敗しました");
                AppError::ChannelCreationFailed
            })?;

        let message_id = send_time_selection_message(gateway, new_channel_id).await?;

        setup_service
            .create_date_channel(
                txn,
                guild_id as i64,
                new_channel_id.get() as i64,
                date.month() as i32,
                date.day() as i32,
                i,
                true,
                Some(message_id.get() as i64),
            )
            .await?;

        created_channels += 1;
        debug!(
            channel_id = new_channel_id.get(),
            channel_name, "日時チャンネルを作成しました"
        );
    }

    Ok(created_channels)
}
