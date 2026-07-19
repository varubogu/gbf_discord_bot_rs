use super::common::build_setup_service;
use crate::gateway::{DiscordChannelGateway, DiscordMessageGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentQuestMessageRepository,
    AutoRecruitmentRepository, QuestMatchingRepository, QuestMatchingUserRepository,
};
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::CategorySetupService;
use crate::types::discord::{DiscordChannelId, DiscordMessageId};
use crate::types::{AppError, AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// カテゴリの自動募集を解除
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `command_channel_id` - コマンド実行チャンネルID
///
/// # エラー
/// * カテゴリ内のチャンネルでコマンドが実行された場合、`InCategoryChannelError`を返す
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn unregister_category<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    command_channel_id: u64,
) -> Result<()>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    info!(guild_id, "自動募集カテゴリを解除します");

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

        // コマンド実行チャンネルがカテゴリ内かどうかを判定
        let command_channel = DiscordChannelId::new(command_channel_id);
        if let Ok(channel_data) = gateway.get_channel(command_channel).await
            && let Some(parent_id) = channel_data.parent_id
            && parent_id.get() == auto_recruitment.category_id as u64
        {
            return Err(AppError::InCategoryChannelError);
        }

        // マッチングチャンネルの処理
        remove_matching_channel(
            gateway,
            auto_recruitment.matching_channel_id,
            auto_recruitment.matching_channel_is_bot_created,
            auto_recruitment.matching_message_id,
        )
        .await;

        // クエストチャンネルの処理
        remove_quest_channel(
            gateway,
            &setup_service,
            &txn,
            guild_id,
            auto_recruitment.quest_channel_id,
            auto_recruitment.quest_channel_is_bot_created,
        )
        .await?;

        // クエストメッセージのDBレコードを削除
        setup_service
            .delete_all_quest_messages(&txn, guild_id as i64)
            .await?;

        // 日時チャンネルの処理
        remove_date_channels(gateway, &setup_service, &txn, guild_id).await?;

        // マッチング関連データを削除（外部キー制約順）
        setup_service
            .delete_all_matching_data(&txn, guild_id as i64)
            .await?;

        // DBから削除
        setup_service
            .delete_all_date_channels(&txn, guild_id as i64)
            .await?;
        setup_service
            .delete_auto_recruitment(&txn, guild_id as i64)
            .await?;

        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            txn.commit().await?;
            info!(guild_id, "自動募集カテゴリを解除しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, "自動募集カテゴリの解除に失敗しました");
            Err(e)
        }
    }
}

/// マッチングチャンネルを削除する（Bot作成チャンネルは削除、指定チャンネルはメッセージのみ削除）
async fn remove_matching_channel<G>(
    gateway: &G,
    matching_channel_id: Option<i64>,
    is_bot_created: bool,
    message_id: Option<i64>,
) where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    if let Some(matching_ch_id) = matching_channel_id {
        let channel_id = DiscordChannelId::new(matching_ch_id as u64);
        if is_bot_created {
            // Bot作成チャンネルは削除
            if let Err(e) = gateway.delete_channel(channel_id).await {
                error!(
                    channel_id = matching_ch_id,
                    error = %e,
                    "マッチングチャンネルの削除に失敗しました"
                );
            }
        } else if let Some(msg_id) = message_id {
            // 指定チャンネルはメッセージのみ削除
            let message_id = DiscordMessageId::new(msg_id as u64);
            if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                error!(
                    channel_id = matching_ch_id,
                    message_id = msg_id,
                    error = %e,
                    "マッチングチャンネルのメッセージ削除に失敗しました"
                );
            }
        }
    }
}

/// クエストチャンネルを削除する（Bot作成チャンネルは削除、指定チャンネルは各クエストメッセージを削除）
async fn remove_quest_channel<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
    quest_channel_id: Option<i64>,
    is_bot_created: bool,
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
    if let Some(quest_ch_id) = quest_channel_id {
        let channel_id = DiscordChannelId::new(quest_ch_id as u64);
        if is_bot_created {
            // Bot作成チャンネルは削除
            if let Err(e) = gateway.delete_channel(channel_id).await {
                error!(
                    channel_id = quest_ch_id,
                    error = %e,
                    "クエストチャンネルの削除に失敗しました"
                );
            }
        } else {
            // 指定チャンネルは各クエストメッセージを削除
            let quest_messages = setup_service
                .find_quest_messages(txn, guild_id as i64)
                .await?;

            for quest_msg in quest_messages {
                let message_id = DiscordMessageId::new(quest_msg.message_id as u64);
                if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                    error!(
                        channel_id = quest_ch_id,
                        message_id = quest_msg.message_id,
                        quest_id = quest_msg.quest_id,
                        error = %e,
                        "クエストメッセージの削除に失敗しました"
                    );
                }
            }
        }
    }

    Ok(())
}

/// 日時チャンネルを全て削除する（Bot作成チャンネルは削除、指定チャンネルはメッセージのみ削除）
async fn remove_date_channels<G, AR, AC, Q, AQM, QMU, QM, ST>(
    gateway: &G,
    setup_service: &CategorySetupService<AR, AC, Q, AQM, QMU, QM, ST>,
    txn: &sea_orm::DatabaseTransaction,
    guild_id: u64,
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
    let channels = setup_service
        .find_date_channels(txn, guild_id as i64)
        .await?;

    for channel in channels {
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
    }

    Ok(())
}
