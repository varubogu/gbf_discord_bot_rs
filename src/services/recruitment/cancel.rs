//! 募集キャンセル処理のサービス層
//!
//! 純粋なビジネスロジックのみを提供する。
//! Discord API操作はGateway経由またはfacade層で行う。

use sea_orm::{ConnectionTrait, DatabaseTransaction};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::gateway::DiscordMessageGateway;
use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::{GuildMessageTextRepository, MessageTextRepository};
use crate::services::message::MessageService;
use crate::services::message::MessageTextId;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use crate::types::domain_interface_result::CanCancelResult;
use crate::types::{AppError, Result};

/// キャンセル操作の実行者情報
pub struct CancelInvokerContext {
    /// 操作を実行するユーザーのID
    pub user_id: u64,
    /// 実行者が gbf_bot_control ロールを保持しているか
    pub has_bot_control: bool,
}

/// キャンセル可能性をチェックし、結果を返す
///
/// # Arguments
/// * `gateway` - Discord Gateway（メッセージ存在確認用）
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `battle_recruitment_repo` - 募集リポジトリ
/// * `txn` - データベーストランザクション
/// * `invoker` - 操作を実行するユーザーの情報
pub async fn check_can_cancel_recruitment<R: crate::repository::BattleRecruitmentsRepository>(
    gateway: &dyn DiscordMessageGateway,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    battle_recruitment_repo: &R,
    txn: &DatabaseTransaction,
    invoker: CancelInvokerContext,
) -> Result<CanCancelResult> {
    info!(
        "キャンセル可能性チェック開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    // DBから募集情報を取得（エラーの場合はNone扱い）（トランザクション対応版を使用）
    let recruitment_opt = battle_recruitment_repo
        .get_by_message_with_txn(
            txn,
            DiscordGuildId::new(guild_id),
            DiscordChannelId::new(channel_id),
            DiscordMessageId::new(message_id),
        )
        .await?;

    // Gateway経由でDiscordからメッセージを取得（エラーの場合はNone扱い）
    let discord_message_opt = gateway
        .get_message(
            DiscordChannelId::new(channel_id),
            DiscordMessageId::new(message_id),
        )
        .await;

    let result = match (recruitment_opt, discord_message_opt) {
        // DBあり + メッセージあり
        (Some(recruitment), Ok(_message)) => {
            if recruitment.is_canceled {
                CanCancelResult::AlreadyCancelled
            } else {
                // 開催日時を過ぎているかチェック
                let now = chrono::Utc::now();
                if recruitment.quest_start_at <= now {
                    CanCancelResult::EventDatePassed
                } else {
                    // 権限チェック: 募集主本人または gbf_bot_control ロール保持者のみキャンセル可能
                    // host_discord_user_id == 0 は旧データ（作成者不明）を表す
                    let is_owner = recruitment.host_discord_user_id != 0
                        && recruitment.host_discord_user_id == invoker.user_id;
                    if !is_owner && !invoker.has_bot_control {
                        CanCancelResult::PermissionDenied
                    } else {
                        CanCancelResult::Success
                    }
                }
            }
        }
        // DBあり + メッセージなし
        (Some(_), Err(_)) => CanCancelResult::MessageDeleted,
        // DBなし + メッセージあり
        (None, Ok(_)) => CanCancelResult::NotRecruitMessage,
        // DBなし + メッセージなし
        (None, Err(_)) => CanCancelResult::NotFound,
    };

    info!("キャンセル可能性チェック完了: {:?}", result);
    Ok(result)
}

/// メッセージIDから募集をキャンセルする
pub async fn cancel_recruitment_by_message<R: crate::repository::BattleRecruitmentsRepository>(
    txn: &DatabaseTransaction,
    battle_recruitment_repo: &R,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    cancel_message_id: DiscordMessageId,
) -> Result<BattleRecruitments> {
    info!("cancel_recruitment_by_message - キャンセル処理開始");

    // 募集情報の存在確認（トランザクション対応版を使用）
    let recruitment = get_recruitment_from_database(
        guild_id,
        channel_id,
        message_id,
        battle_recruitment_repo,
        txn,
    )
    .await?;

    // 募集をキャンセル済み状態に更新
    mark_recruitment_as_cancelled(
        txn,
        recruitment.id,
        cancel_message_id,
        battle_recruitment_repo,
    )
    .await?;

    info!(recruitment_id = recruitment.id, "キャンセル処理完了");
    Ok(recruitment)
}

/// DBから募集情報を取得
pub async fn get_recruitment_from_database<R: crate::repository::BattleRecruitmentsRepository>(
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    battle_recruitment_repo: &R,
    txn: &DatabaseTransaction,
) -> Result<BattleRecruitments> {
    info!(
        "DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    // u64をドメイン型に変換
    match battle_recruitment_repo
        .get_by_message_with_txn(
            txn,
            DiscordGuildId::new(guild_id),
            DiscordChannelId::new(channel_id),
            DiscordMessageId::new(message_id),
        )
        .await?
    {
        Some(recruitment) => {
            info!("募集情報取得成功: recruitment_id={}", recruitment.id);
            Ok(recruitment)
        }
        None => {
            warn!("募集情報が見つかりません: message_id={}", message_id);
            Err(AppError::Business {
                message: format!("Recruitment not found for message_id: {message_id}"),
            })
        }
    }
}

/// 募集をキャンセル済み状態に更新
pub async fn mark_recruitment_as_cancelled<R: crate::repository::BattleRecruitmentsRepository>(
    txn: &DatabaseTransaction,
    recruitment_id: i32,
    cancel_message_id: DiscordMessageId,
    battle_recruitment_repo: &R,
) -> Result<()> {
    info!(
        "募集キャンセル済み状態更新: recruitment_id={}",
        recruitment_id
    );

    // 終了メッセージID = 0 でキャンセル状態を表現
    battle_recruitment_repo
        .set_canceled_with_txn(txn, recruitment_id, cancel_message_id)
        .await?;

    info!(
        "募集キャンセル済み状態更新完了: recruitment_id={}",
        recruitment_id
    );
    Ok(())
}

/// キャンセル済みメッセージ作成
pub async fn create_cancelled_message_content<C, GM, MT>(
    db: &C,
    message_service: &MessageService<GM, MT>,
    guild_id: Option<i64>,
    locale: Option<&str>,
    original_content: &str,
) -> Result<String>
where
    C: ConnectionTrait,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
{
    // メッセージサービスからキャンセル済みサフィックスを取得
    let cancelled_suffix = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentCommandCancelledMessageSuffix.as_str(),
            HashMap::new(),
            guild_id,
            locale,
        )
        .await
        .unwrap_or_else(|_| "この募集はキャンセルされました".to_string());

    // 元のメッセージに打ち消し線と「キャンセル済み」を追加
    Ok(format!("~~{original_content}~~\n\n**{cancelled_suffix}**"))
}

/// キャンセル通知メッセージ作成
pub async fn create_cancel_notification_text<C, GM, MT>(
    db: &C,
    message_service: &MessageService<GM, MT>,
    guild_id: Option<i64>,
    locale: Option<&str>,
    participant_user_ids: &[u64],
) -> Result<String>
where
    C: ConnectionTrait,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
{
    if participant_user_ids.is_empty() {
        // 参加者なしの場合
        let message = message_service
            .get_message(
                db,
                MessageTextId::RecruitmentCommandCancelNotificationNoParticipants.as_str(),
                HashMap::new(),
                guild_id,
                locale,
            )
            .await
            .unwrap_or_else(|_| "募集がキャンセルされました。".to_string());
        Ok(message)
    } else {
        // 参加者ありの場合
        let base_message = message_service
            .get_message(
                db,
                MessageTextId::RecruitmentCommandCancelNotificationWithParticipants.as_str(),
                HashMap::new(),
                guild_id,
                locale,
            )
            .await
            .unwrap_or_else(|_| "募集がキャンセルされました。\n参加予定だった皆さん".to_string());

        let participants_str = participant_user_ids
            .iter()
            .map(|user_id| format!("<@{user_id}>"))
            .collect::<Vec<_>>()
            .join(" ");
        Ok(format!("{base_message}: {participants_str}"))
    }
}
