use crate::gateway::DiscordMessageGateway;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_update_service::RecruitmentUpdateService;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, MessageContent};
use crate::types::{AppState, Result};
use tracing::{info, instrument};

/// 規定人数到達の通知処理
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `txn` - データベーストランザクション
/// * `recruitment` - 募集情報
/// * `participant_count` - 現在の参加者数
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
#[instrument(level = "info", skip(gateway, app_state, txn))]
pub(super) async fn check_and_notify_recruitment_full<G>(
    gateway: &G,
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment: &crate::models::battle_recruitments::BattleRecruitments,
    participant_count: usize,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    info!("規定人数到達チェックを開始します");

    // クエスト情報を取得して規定人数を確認
    let quest_query_service = QuestQueryService::new(app_state.repositories.quest);
    let quest = quest_query_service
        .get_quest_by_id(txn, recruitment.quest_id)
        .await?;

    let required_count = quest.recruit_count as usize;
    let is_full = participant_count >= required_count;
    let notification_sent = recruitment.full_notification_sent;

    info!(
        participant_count = participant_count,
        required_count = required_count,
        is_full = is_full,
        notification_sent = notification_sent,
        "人数チェック結果"
    );

    let recruitment_update_service =
        RecruitmentUpdateService::new(app_state.repositories.battle_recruitments);

    match (notification_sent, is_full) {
        (false, false) => {
            // フラグ無し（未送信）で規定人数未満 → 何もしない
            info!("規定人数未達のため通知しません");
            Ok(())
        }
        (false, true) => {
            // フラグ無し（未送信）で規定人数以上 → フラグを立てて通知送信
            info!("規定人数に到達しました。通知を送信します");

            // 全参加者のメンションを取得
            let participants = get_all_participant_mentions(app_state, txn, recruitment.id).await?;

            // 通知メッセージを送信
            send_full_notification(gateway, channel_id, message_id, participants).await?;

            // フラグを立てる
            recruitment_update_service
                .set_full_notification_sent(txn, recruitment.id, true)
                .await?;

            info!("規定人数到達通知を送信しました");
            Ok(())
        }
        (true, false) => {
            // フラグあり（送信済）で規定人数未満 → フラグを下げて減少通知送信
            info!("参加者が規定人数を下回りました。通知を送信します");

            // 減少通知メッセージを送信
            send_decreased_notification(gateway, channel_id, message_id).await?;

            // フラグを下げる
            recruitment_update_service
                .set_full_notification_sent(txn, recruitment.id, false)
                .await?;

            info!("参加者減少通知を送信しました");
            Ok(())
        }
        (true, true) => {
            // フラグあり（送信済）で規定人数以上 → 何もしない
            info!("既に通知済みで規定人数以上のため何もしません");
            Ok(())
        }
    }
}

/// 全参加者のメンションを取得
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `recruitment_id` - 募集ID
async fn get_all_participant_mentions(
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment_id: i32,
) -> Result<Vec<String>> {
    use std::collections::HashSet;

    let participants_service =
        RecruitmentParticipantsService::new(app_state.repositories.recruitment_participants);
    let participants = participants_service
        .find_by_recruitment_id(txn, recruitment_id)
        .await?;

    // ユニークなユーザーIDを取得（重複排除）
    let unique_user_ids: HashSet<i64> = participants.iter().map(|p| p.user_id).collect();

    Ok(unique_user_ids
        .into_iter()
        .map(|user_id| format!("<@{user_id}>"))
        .collect())
}

/// 規定人数到達通知メッセージを送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `participants` - 参加者のメンション一覧
async fn send_full_notification<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
    participants: Vec<String>,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    let notification_message = format!("{}\n参加人数が集まりました。", participants.join(" "));

    // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
    gateway
        .send_reply(
            channel_id,
            message_id,
            MessageContent::text(&notification_message),
            Some("規定人数到達通知".to_string()),
        )
        .await?;

    Ok(())
}

/// 参加者減少通知メッセージを送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
async fn send_decreased_notification<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    let notification_message = "参加メンバーが規定人数を下回りました。";

    // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
    gateway
        .send_reply(
            channel_id,
            message_id,
            MessageContent::text(notification_message),
            Some("参加者減少通知".to_string()),
        )
        .await?;

    Ok(())
}
