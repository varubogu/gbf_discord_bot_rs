use crate::gateway::DiscordReactionGateway;
use crate::types;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, MessageData};
use tracing::{error, info};

/// 通知向け参加者ユーザーIDを収集する。
///
/// DB参加者とリアクション参加者を合算し、重複を除去して返す。
pub async fn collect_notification_participant_user_ids<G>(
    mut participant_user_ids: Vec<u64>,
    gateway: &G,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
    message: &MessageData,
) -> types::Result<Vec<u64>>
where
    G: DiscordReactionGateway + Sync,
{
    for reaction in &message.reactions {
        match gateway
            .get_reaction_users(channel_id, message_id, reaction.emoji.clone(), Some(100))
            .await
        {
            Ok(user_ids) => {
                participant_user_ids.extend(user_ids.into_iter().map(|user_id| user_id.get()));
            }
            Err(e) => {
                error!(
                    error = %e,
                    channel_id = %channel_id.get(),
                    message_id = %message_id.get(),
                    "リアクション参加者の取得に失敗しました（DB参加者で処理を継続します）"
                );
            }
        }
    }

    participant_user_ids.sort_unstable();
    participant_user_ids.dedup();

    info!(
        participants_count = participant_user_ids.len(),
        "通知向け参加者ユーザーIDを収集しました"
    );

    Ok(participant_user_ids)
}

/// ユーザーID一覧をDiscordメンション形式へ変換する。
pub fn to_mentions(participant_user_ids: &[u64]) -> Vec<String> {
    participant_user_ids
        .iter()
        .map(|user_id| format!("<@{user_id}>"))
        .collect()
}
