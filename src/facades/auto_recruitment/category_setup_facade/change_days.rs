use super::common::build_setup_service;
use super::messages::send_time_selection_message;
use crate::gateway::{DiscordChannelGateway, DiscordMessageGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::entities::guild_master::auto_recruitment_channels;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentQuestMessageRepository,
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::CategorySetupService;
use crate::types::discord::{
    ChannelCreateParams, ChannelEditParams, DiscordChannelId, DiscordGuildId, DiscordMessageId,
};
use crate::types::{AppError, AppState, Result};
use crate::utils::datetime_display::format_date_channel_name_ja;
use chrono::{Datelike, Duration, Utc};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 募集日数を変更
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `new_days` - 新しい募集日数（2-7日）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn change_days<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    new_days: i32,
) -> Result<()>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    info!(guild_id, new_days, "自動募集の募集日数を変更します");

    // 日数の検証
    if !(2..=7).contains(&new_days) {
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

        // 自動募集設定を取得
        let auto_recruitment = setup_service
            .get_auto_recruitment_or_err(&txn, guild_id as i64)
            .await?;

        let current_days = auto_recruitment.days_range;

        if new_days == current_days {
            return Err(AppError::Business {
                message: format!("募集日数は既に{new_days}日です"),
            });
        }

        // 既存のチャンネルを取得
        let existing_channels = setup_service
            .find_date_channels(&txn, guild_id as i64)
            .await?;

        let category_id = auto_recruitment.category_id as u64;

        if new_days > current_days {
            increase_date_channels(
                gateway,
                &setup_service,
                &txn,
                guild_id,
                category_id,
                &existing_channels,
                current_days,
                new_days,
                auto_recruitment.quest_channel_id,
            )
            .await?;
        } else {
            decrease_date_channels(
                gateway,
                &setup_service,
                &txn,
                guild_id,
                &existing_channels,
                (current_days - new_days) as usize,
            )
            .await?;
        }

        // 日数を更新
        setup_service
            .update_days_range(&txn, guild_id as i64, new_days)
            .await?;

        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            txn.commit().await?;
            info!(guild_id, new_days, "自動募集の募集日数を変更しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, "自動募集の募集日数の変更に失敗しました");
            Err(e)
        }
    }
}

/// 募集日数を増やす場合：末尾に追加の日時チャンネルを作成し、クエストチャンネルの位置を更新する
#[allow(clippy::too_many_arguments)]
async fn increase_date_channels<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
    category_id: u64,
    existing_channels: &[auto_recruitment_channels::Model],
    current_days: i32,
    new_days: i32,
    quest_channel_id: Option<i64>,
) -> Result<()>
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
    // 増やす場合：追加のチャンネルを作成
    let channels_to_add = (new_days - current_days) as usize;
    let last_channel_date = if let Some(last) = existing_channels.last() {
        let now_jst = Utc::now() + Duration::hours(9);
        let today = now_jst.date_naive();
        chrono::NaiveDate::from_ymd_opt(today.year(), last.month as u32, last.day as u32)
            .unwrap_or(today)
    } else {
        let now_jst = Utc::now() + Duration::hours(9);
        now_jst.date_naive()
    };

    let discord_guild_id = DiscordGuildId::new(guild_id);

    for i in 1..=channels_to_add {
        let new_date = last_channel_date + Duration::days(i as i64);
        let channel_name = format_date_channel_name_ja(new_date);
        let sort_order = (existing_channels.len() + i) as i32;
        // 日付チャンネルはposition 1から（position 0はマッチング）
        let channel_position = (existing_channels.len() + i) as u16;

        // Discordチャンネルを作成（カテゴリの権限を継承、位置指定）
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

        // 時間選択コンポーネントを送信してメッセージIDを取得
        let message_id = send_time_selection_message(gateway, new_channel_id).await?;

        // DBに登録（Bot作成フラグ=true、メッセージID保存）
        setup_service
            .create_date_channel(
                txn,
                guild_id as i64,
                new_channel_id.get() as i64,
                new_date.month() as i32,
                new_date.day() as i32,
                sort_order,
                true,
                Some(message_id.get() as i64),
            )
            .await?;
    }

    // クエストチャンネルの位置を日付チャンネルの後に更新
    if let Some(quest_ch_id) = quest_channel_id {
        let quest_channel = DiscordChannelId::new(quest_ch_id as u64);
        let _ = gateway
            .edit_channel(
                quest_channel,
                ChannelEditParams::new().with_position((new_days + 1) as u16),
            )
            .await;
    }

    Ok(())
}

/// 募集日数を減らす場合：末尾の日時チャンネルを削除する
async fn decrease_date_channels<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
    existing_channels: &[auto_recruitment_channels::Model],
    channels_to_remove: usize,
) -> Result<()>
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
    let channels_to_delete: Vec<_> = existing_channels
        .iter()
        .rev()
        .take(channels_to_remove)
        .collect();

    for channel in channels_to_delete {
        let channel_id = DiscordChannelId::new(channel.channel_id as u64);
        if channel.is_bot_created {
            // Bot作成チャンネルは削除
            if let Err(e) = gateway.delete_channel(channel_id).await {
                error!(
                    channel_id = channel.channel_id,
                    error = %e,
                    "日時チャンネルの削除に失敗しました"
                );
            }
        } else if let Some(msg_id) = channel.message_id {
            // 指定チャンネルはメッセージのみ削除
            let message_id = DiscordMessageId::new(msg_id as u64);
            if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                error!(
                    channel_id = channel.channel_id,
                    message_id = msg_id,
                    error = %e,
                    "日時チャンネルのメッセージ削除に失敗しました"
                );
            }
        }

        // DBから削除
        setup_service
            .delete_date_channel_by_channel_id(txn, guild_id as i64, channel.channel_id)
            .await?;
    }

    Ok(())
}
